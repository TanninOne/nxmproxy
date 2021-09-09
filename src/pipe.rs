use std::{fs::File, io::{Error, Write}};

fn write_pipe(pipe: &str, value: &str) -> Result<usize, Error> {
    return File::create(format!(r"\\.\pipe\{}", pipe))?.write(value.as_bytes());
}

pub fn try_write_pipe(pipe: &str, value: &str) -> bool {
    match write_pipe(pipe, value) {
        Ok(_) => true,
        Err(_) => false,
    }
}
