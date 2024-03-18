use std::{
    cmp,
    io::{stdout, Write},
};

use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Print, SetBackgroundColor, SetForegroundColor},
};

use crate::gap_buffer::TextGapBuffer;
use crate::logger;

pub enum Direction {
    LEFT,
    DOWN,
    UP,
    RIGHT,
}

pub struct Cursor {
    row: u16,
    column: u16,
}

pub struct LineInfo {
    pub index: usize,
    pub tabs: Vec<u16>,
    pub len: u16,
}

pub struct LineChunk {
    len: u16,
    end_of_line: bool,
    tabs: Vec<u16>,
}

pub struct ScreenDimensions {
    pub row: u16,
    pub column: u16,
    pub max_rows: u16,
    pub max_cols: u16,
}

pub struct Editor {
    cursor: Cursor,
    window_dim: ScreenDimensions,
    editor_dim: ScreenDimensions,
    gap_buffer: TextGapBuffer,

    title: String,
    line_offset: usize,
    line_map: Vec<LineInfo>,
}

impl Editor {
    pub fn new(window_dim: ScreenDimensions, title: String) -> Self {
        let mut gap_buffer = TextGapBuffer::new();
        //let basic_string = "abc\tcde\tfgh\nxxxxxxxxxxxxxxxxx";
        let basic_string = "But I must explain to you how all this mistaken idea of denouncing pleasure and praising pain was born and I will give you a complete account of the system, and expound the actual teachings of the great explorer of the truth, the master-builder of human happiness. No one rejects, dislikes, or avoids pleasure itself, because it is pleasure, but because those who do not know how to pursue pleasure rationally encounter consequences that are extremely painful. Nor again is there anyone who loves or pursues or desires to obtain pain of itself, because it is pain, but because occasionally circumstances occur in which toil and pain can procure him some great pleasure. To take a trivial example, which of us ever undertakes laborious physical exercise, except to obtain some advantage from it?\nBut who has any right to find fault with a man who chooses to enjoy a pleasure that has no annoying consequences, or one who avoids a pain that produces no resultant pleasure?";
        let cols_limit = window_dim.max_cols - 4;

        for i in 0..basic_string.len() {
            gap_buffer.insert_ch(basic_string.chars().nth(i).unwrap())
        }

        let editor_dim = ScreenDimensions {
            row: window_dim.row + 2,
            column: window_dim.column + 2,
            max_rows: window_dim.max_rows - 4,
            max_cols: window_dim.max_cols - 4,
        };

        let mut instance = Self {
            cursor: Cursor { row: 0, column: 0 },
            window_dim,
            editor_dim,
            gap_buffer,

            title,
            line_offset: 0,
            line_map: Vec::new(),
        };

        instance.line_map = instance.get_line_map(cols_limit);
        instance.draw_window();
        instance.draw_lines(0);
        instance.move_to_cursor();
        instance
    }

    pub fn get_line_chunk(self: &Self, start: usize, limit: u16) -> LineChunk {
        let mut last_space = 0;

        let mut i = start;
        let mut tabs = Vec::new();
        let mut limit = limit as usize;
        while limit > 0 {
            let ch = match self.gap_buffer.get(i) {
                Ok(ch) => ch,
                Err(_) => {
                    return LineChunk {
                        len: (i - start) as u16,
                        end_of_line: false,
                        tabs,
                    }
                }
            };

            match ch {
                '\n' => {
                    return LineChunk {
                        len: (i - start) as u16,
                        end_of_line: true,
                        tabs,
                    }
                }
                '\t' => {
                    tabs.push((i - start) as u16);
                    limit -= 4;
                    i += 1;
                    continue;
                }
                ' ' => last_space = i,
                _ => {}
            }

            i += 1;
            limit -= 1;
        }

        let ch = self.gap_buffer.get(i - 1).unwrap();
        if i < self.gap_buffer.len() - 1 && ch != ' ' {
            return LineChunk {
                len: (last_space - start + 1) as u16,
                end_of_line: false,
                tabs,
            };
        }

        return LineChunk {
            len: (i - start) as u16,
            end_of_line: false,
            tabs,
        };
    }

    pub fn get_line_map(self: &Self, limit: u16) -> Vec<LineInfo> {
        let mut index = 0;
        let mut vec: Vec<LineInfo> = Vec::new();

        while index < self.gap_buffer.len() {
            let line_chunk = self.get_line_chunk(index, limit);
            vec.push(LineInfo {
                index,
                len: line_chunk.len,
                tabs: line_chunk.tabs.clone(),
            });
            index += line_chunk.len as usize + (if line_chunk.end_of_line { 1 } else { 0 });
            if line_chunk.end_of_line && index >= self.gap_buffer.len() {
                vec.push(LineInfo {
                    index,
                    len: 0,           //line_chunk.len,
                    tabs: Vec::new(), //line_chunk.tabs.clone()
                });
            }
        }

        return vec;
    }

    fn get_tab_rectified(self: &Self, line: &LineInfo, column: u16) -> u16 {
        let tabs = &line.tabs;
        for i in 0..tabs.len() {
            let tab = tabs[i] + (i as u16 * 3);
            if column >= tab && column < tab + (4 / 2) {
                return tab;
            } else if column >= tab + (4 / 2) && column < tab + 4 {
                return tab + 4;
            }
        }

        column
    }

    fn get_rel_cursor(self: &Self) -> Cursor {
        let line = &self.line_map[self.line_offset + (self.cursor.row as usize)];
        let column = std::cmp::min(
            self.cursor.column,
            line.len + (line.tabs.len() as u16 * 3), //tab_size - 1
        );

        logger::log(format!("get_rel_cursor: {}", column).as_str());
        Cursor {
            row: self.cursor.row,
            column: self.get_tab_rectified(line, column),
        }
    }

    fn get_current_index(self: &Self) -> usize {
        let rel_cursor = self.get_rel_cursor();

        let line_index = self.line_offset + rel_cursor.row as usize;
        let line = &self.line_map[line_index];

        let tabs = &line.tabs;
        for i in 0..tabs.len() {
            let col = rel_cursor.column - (i as u16 * 3);
            if col < tabs[i] {
                return line.index + col as usize;
            } else if col >= tabs[i] && col < tabs[i] + (4 / 2) {
                return line.index + tabs[i] as usize;
            } else if col >= tabs[i] + (4 / 2) && col < tabs[i] + 4 {
                return line.index + tabs[i] as usize + 1;
            }
        }

        line.index + rel_cursor.column as usize - ((4 - 1) * tabs.len())
    }

    fn get_index_line(self: &Self, index: usize) -> usize {
        for i in 0..(self.line_map.len() - 1) {
            let line = &self.line_map[i];
            let next_line = &self.line_map[i + 1];
            if index >= line.index && index < next_line.index {
                return i;
            }
        }

        return self.line_map.len() - 1;
    }

    fn get_cursor_from_index(self: &Self, index: usize) -> Cursor {
        let start_line = self.line_offset;
        let line_index = self.get_index_line(index);
        let line = &self.line_map[line_index];

        let mut found_tabs = 0;
        for i in 0..line.tabs.len() {
            if line.index + line.tabs[i] as usize >= index {
                break;
            }

            found_tabs += 1;
        }

        logger::log(
            format!(
                "get_cursor_from_index: {} -> {}",
                index,
                (index + (found_tabs * 3)) - line.index
            )
            .as_str(),
        );

        return Cursor {
            row: (line_index - start_line) as u16,
            column: ((index + (found_tabs * 3)) - line.index) as u16,
        };
    }

    fn move_to_cursor(self: &mut Self) {
        //TODO: Adjust line_offset accordingly to accomodate the cursor.
        let rel_cursor = self.get_rel_cursor();
        if rel_cursor.row > self.editor_dim.max_rows {
            let curr_index = self.get_current_index();
            let cursor = self.get_cursor_from_index(curr_index);
            self.line_offset = (self.line_offset + cursor.row as usize) - self.editor_dim.max_rows as usize;
            self.cursor.row = self.editor_dim.max_rows - 2;
        }

        logger::log(
            format!(
                "Cursor: [row: {}, column: {}]",
                rel_cursor.row, rel_cursor.column
            )
            .as_str(),
        );
        execute!(
            stdout(),
            MoveTo(
                rel_cursor.column + self.editor_dim.column,
                rel_cursor.row + self.editor_dim.row
            )
        )
        .unwrap();
    }

    //TODO: Handle line offset, here itself
    pub fn move_cursor(self: &mut Self, direction: Direction, magnitude: u16) {
        let mut redraw_lines = false;

        match direction {
            Direction::LEFT => {
                let index = self.get_current_index();
                logger::log(format!("Index: {}", index).as_str());
                if index as isize - 1 >= 0 {
                    let cur = self.get_cursor_from_index(index - 1);
                    self.cursor.row = cur.row;
                    self.cursor.column = cur.column;
                }
            }
            Direction::DOWN => {
                let line_offset_max_limit = cmp::min(
                    self.editor_dim.max_rows as usize,
                    self.line_map.len() - self.line_offset,
                );

                logger::log(
                    format!(
                        "[BEFORE] cursor.row = {}, rows = {}, line_map = {}",
                        self.cursor.row,
                        self.editor_dim.max_rows,
                        self.line_map.len()
                    )
                    .as_str(),
                );
                self.cursor.row = if self.cursor.row + 1 == (line_offset_max_limit as u16) {
                    if (self.line_offset + self.editor_dim.max_rows as usize) < self.line_map.len()
                    {
                        redraw_lines = true;
                        self.line_offset += 1;
                        logger::log(format!("Line Offset incremented").as_str());
                    }
                    line_offset_max_limit as u16 - 1
                } else {
                    logger::log(format!("cursor incremented").as_str());
                    self.cursor.row + 1
                };
                logger::log(format!("[AFTER] cursor.row = {}", self.cursor.row).as_str());
            }
            Direction::UP => {
                self.cursor.row = if self.cursor.row == 0 {
                    if self.line_offset > 0 {
                        redraw_lines = true;
                        self.line_offset -= 1;
                    }
                    0
                } else {
                    self.cursor.row - 1
                };
            }
            Direction::RIGHT => {
                let index = self.get_current_index();
                logger::log(format!("Index: {}", index).as_str());
                if index < self.gap_buffer.len() {
                    let cur = self.get_cursor_from_index(index + 1);
                    self.cursor.row = cur.row;
                    self.cursor.column = cur.column;
                }
            }
        }

        if redraw_lines {
            self.draw_lines(0);
        }

        self.move_to_cursor();
    }

    fn draw_window(self: &Self) {
        queue!(
            stdout(),
            SetBackgroundColor(crossterm::style::Color::Rgb {
                r: 17,
                g: 18,
                b: 29
            }),
            SetForegroundColor(crossterm::style::Color::Rgb {
                r: 238,
                g: 109,
                b: 133
            }),
        )
        .unwrap();

        for i in 0..self.window_dim.max_rows {
            queue!(
                stdout(),
                MoveTo(self.window_dim.column, self.window_dim.row + i),
                Print(
                    (0..self.window_dim.max_cols)
                        .map(|_| " ")
                        .collect::<String>()
                        .as_str()
                ),
            )
            .unwrap();
        }

        queue!(
            stdout(),
            MoveTo(self.window_dim.column, self.window_dim.row),
            Print(
                (0..self.window_dim.max_cols)
                    .map(|_| " ")
                    .collect::<String>()
                    .as_str()
            ),
            MoveTo(self.editor_dim.column, self.window_dim.row),
            Print(self.title.clone()),
            MoveTo(
                self.window_dim.column,
                self.window_dim.row + self.window_dim.max_rows
            ),
            Print(
                (0..self.window_dim.max_cols)
                    .map(|_| " ")
                    .collect::<String>()
                    .as_str()
            ),
            MoveTo(
                self.editor_dim.column,
                self.window_dim.row + self.window_dim.max_rows
            ),
            Print("==="),
        )
        .unwrap();

        stdout().flush().unwrap();
    }

    fn draw_lines(self: &Self, start_line: u16) {
        queue!(
            stdout(),
            SetBackgroundColor(crossterm::style::Color::Rgb {
                r: 17,
                g: 18,
                b: 29
            }),
            SetForegroundColor(crossterm::style::Color::White),
        )
        .unwrap();

        for i in 0..self.editor_dim.max_rows {
            queue!(
                stdout(),
                MoveTo(self.editor_dim.column, self.editor_dim.row + i),
                Print(
                    (0..self.editor_dim.max_cols)
                        .map(|_| " ")
                        .collect::<String>()
                        .as_str()
                ),
            )
            .unwrap();
        }

        for i in 0..self.editor_dim.max_rows {
            if self.line_offset + (i as usize) >= self.line_map.len() {
                break;
            }

            let line_index = self.line_offset + i as usize;
            let line = &self.line_map[line_index];
            let mut line_str = String::new();
            for i in line.index..(line.index + line.len as usize) {
                let ch = self.gap_buffer.get(i).unwrap();
                if ch == '\t' {
                    for _ in 0..4 {
                        line_str.push(' ');
                    }
                    continue;
                }
                line_str.push(ch);
            }

            queue!(
                stdout(),
                MoveTo(self.editor_dim.column, self.editor_dim.row + (i as u16)),
                Print(
                    (0..self.editor_dim.max_cols)
                        .map(|_| " ")
                        .collect::<String>()
                        .as_str()
                ),
                MoveTo(self.editor_dim.column, self.editor_dim.row + (i as u16)),
                Print(line_str)
            )
            .unwrap();
        }

        stdout().flush().unwrap();
    }

    pub fn insert_ch(self: &mut Self, ch: char) {
        let curr_index = self.get_current_index();
        self.gap_buffer.move_window(curr_index);
        self.gap_buffer.insert_ch(ch);

        self.line_map = self.get_line_map(self.editor_dim.max_cols);

        let new_cursor = self.get_cursor_from_index(curr_index + 1);
        self.cursor.row = new_cursor.row;
        self.cursor.column = new_cursor.column;

        let line_offset_max_limit =
            cmp::min(self.editor_dim.max_rows as usize, self.line_map.len());
        if self.cursor.row == (line_offset_max_limit as u16) {
            if (self.line_offset + self.editor_dim.max_rows as usize) < self.line_map.len() {
                self.cursor.row -= 1;
                self.line_offset += 1;
            }
        }

        self.draw_lines(0);
        self.move_to_cursor();
    }

    pub fn delete_ch(self: &mut Self) {
        let curr_index = self.get_current_index();
        if curr_index == 0 {
            return;
        }

        self.gap_buffer.move_window(curr_index);
        self.gap_buffer.delete_ch();

        self.line_map = self.get_line_map(self.editor_dim.max_cols);
        if self.line_offset > 0
            && self.line_map.len() - self.line_offset < self.editor_dim.max_rows as usize
        {
            self.line_offset -= 1;
        } else {
            let new_cursor = self.get_cursor_from_index(curr_index - 1);
            self.cursor.row = new_cursor.row;
            self.cursor.column = new_cursor.column;
        }

        self.draw_lines(0);
        self.move_to_cursor();
    }

    pub fn resize_redraw(self: &mut Self, window_dim: ScreenDimensions) {
        let index = self.get_current_index();

        self.window_dim.row = window_dim.row;
        self.window_dim.column = window_dim.column;
        self.window_dim.max_rows = window_dim.max_rows;
        self.window_dim.max_cols = window_dim.max_cols;

        self.editor_dim.row = window_dim.row + 2;
        self.editor_dim.column = window_dim.column + 2;
        self.editor_dim.max_rows = window_dim.max_rows - 4;
        self.editor_dim.max_cols = window_dim.max_cols - 4;

        self.line_map = self.get_line_map(self.editor_dim.max_cols);
        if self.line_offset > 0
            && self.line_map.len() - self.line_offset < self.editor_dim.max_rows as usize
        {
            let value = self.line_map.len() as isize - self.editor_dim.max_rows as isize - 1;
            self.line_offset = if value < 0 { 0 } else { value as usize };
        }
        self.draw_window();
        self.draw_lines(0);

        let cursor = self.get_cursor_from_index(index);
        self.cursor.row = cursor.row;
        self.cursor.column = cursor.column;
        self.move_to_cursor();
    }
}
