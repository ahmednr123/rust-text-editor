use std::io::stdout;
use std::time::Duration;
use std::env;
use std::panic;

mod editor;
mod logger;
mod gap_buffer;

use crossterm::{
    execute,
    ExecutableCommand,
    terminal::{Clear, ClearType, EnterAlternateScreen, enable_raw_mode, LeaveAlternateScreen, disable_raw_mode, window_size},
    cursor::{MoveToColumn, MoveToRow, EnableBlinking, DisableBlinking},
    event::{poll, read, Event, KeyCode}
};
use editor::{ScreenDimensions, Direction};

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    env::set_var("RUST_BACKTRACE", "1");

    panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::capture();

        stdout()
            .execute(DisableBlinking).unwrap()
            .execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();

        eprintln!("{}", panic_info);
        println!("{}", backtrace);
    }));

    execute!(
        stdout(),
        EnterAlternateScreen,
        Clear(ClearType::All),
        EnableBlinking
    )?;

    let mut journal = editor::Editor::new(
        ScreenDimensions {
            row: 0,
            column: 0,
            max_rows: window_size()?.rows,
            max_cols: window_size()?.columns
        },
        String::from("[Code Journal]")
    );

    loop {
        if poll(Duration::from_millis(500))? {
            match read()? {
                Event::FocusGained => println!("FocusGained"),
                Event::FocusLost => println!("FocusLost"),
                Event::Key(event) => {
                    if event.code == KeyCode::Char('q') {
                        stdout()
                            .execute(DisableBlinking)?
                            .execute(LeaveAlternateScreen)?;
                        disable_raw_mode()?;
                        break;
                    }

                    match event.code {
                        KeyCode::Left       => journal.move_cursor(Direction::LEFT, 1),
                        KeyCode::Down       => journal.move_cursor(Direction::DOWN, 1),
                        KeyCode::Up         => journal.move_cursor(Direction::UP, 1),
                        KeyCode::Right      => journal.move_cursor(Direction::RIGHT, 1),
                        KeyCode::Char(ch)   => journal.insert_ch(ch),
                        KeyCode::Enter      => journal.insert_ch('\n'),
                        KeyCode::Tab        => journal.insert_ch('\t'),
                        KeyCode::Backspace  => journal.delete_ch(),
                        _ => {}
                    }
                },
                Event::Mouse(event) => println!("{:?}", event),
                Event::Resize(width, height) => {
                    let w_size = window_size()?;
                    //println!("New size {}x{}", width, height);
                    //println!("Crossterm {}x{} {}x{}", w_size.width, w_size.height, w_size.rows, w_size.columns);
                    journal.resize_redraw(ScreenDimensions {
                        row: 0,
                        column: 0,
                        max_rows: w_size.rows,
                        max_cols: w_size.columns
                    });
                },
                _ => println!("Unknown event")
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    };

    Ok(())
}
