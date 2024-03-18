mod journal;
mod logger;
mod gap_buffer;

use gap_buffer::TextGapBuffer;

fn main () {
    let mut gap_buffer = TextGapBuffer::new_in(20, 4);
    gap_buffer.insert_ch('\t');
    gap_buffer.insert_ch('a');
    gap_buffer.insert_ch('b');
    gap_buffer.insert_ch('c');
    gap_buffer.insert_ch('\n');

    let lines = gap_buffer.get_line_map(5).unwrap();
    for i in 0..lines.len() {
        println!("index: {}, len: {}", &lines[i].index, &lines[i].len);
    }

    println!("len: {}", gap_buffer.len());
    println!("len_with_gap: {}", gap_buffer.len_with_gap());
    println!("cap: {}", gap_buffer.cap());

    //gap_buffer.move_window(9);
    //gap_buffer.delete_ch();

    //let lines = gap_buffer.get_line_map(5).unwrap();
    //for i in 0..lines.len() {
    //    println!("index: {}, len: {}", &lines[i].index, &lines[i].len);
    //}

    //println!("len: {}", gap_buffer.len());
    //println!("len_with_gap: {}", gap_buffer.len_with_gap());
    //println!("cap: {}", gap_buffer.cap());
    //let str = gap_buffer.get_string(0, gap_buffer.len());
    //let len = gap_buffer.trim_len(0, 11);
    //println!("len: {}, gap: {}", len.0, gap_buffer.gap_window.window_size);

    //gap_buffer.move_window(0);

    //let str = gap_buffer.get_string(0, gap_buffer.len());
    //let len = gap_buffer.trim_len(0, 11);
    //println!("len: {}, gap: {}", len.0, gap_buffer.gap_window.window_size);
}
