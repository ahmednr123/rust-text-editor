use std::fs::OpenOptions;
use std::io::Write;

pub fn log (data: &str) {
    let mut data_file = OpenOptions::new().append(true)
        .open("log.txt")
        .expect("cannot open file");

    let mut line = String::from(data);
    line.push('\n');
    data_file.write(line.as_bytes())
        .expect("write failed");
}
