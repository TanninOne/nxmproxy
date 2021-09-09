use std::{fs::{File, OpenOptions}, io::{Error, Write}, sync::Mutex};

use once_cell::sync::Lazy;

/// Yes yes, there is a log crate, why the DIY crap?
/// Because the log crate more than doubles the final executable size for some logging
/// that likely noone ever reads, that's why

pub struct Logger {
    file: Option<File>
}

impl Logger {
    pub fn set_file(self: & mut Logger, file_path: &str) {
        self.file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(file_path)
            .ok();
    }

    pub fn log(self: & mut Logger, level: &str, message: &str) -> Result<(), Error> {
        match self.file.as_mut() {
            Some(f) => f.write(format!("{}: {}\n", level, message).as_bytes()).and(Ok(())),
            None => {
                println!("{}: {}", level, message);
                Ok(())
            },
        }
    }
}


static LOGGER: Lazy<Mutex<Logger>> = Lazy::new(|| {
    Mutex::new(Logger { file : None })
});

pub fn set_file(file_path: &str) {
    LOGGER.lock().unwrap().set_file(file_path);
}

#[allow(dead_code)]
pub fn debug<S: AsRef<str>>(message: S) {
    LOGGER.lock().unwrap().log("debug", message.as_ref()).ok();
}

#[allow(dead_code)]
pub fn info<S: AsRef<str>>(message: S) {
    LOGGER.lock().unwrap().log("info", message.as_ref()).ok();
}

#[allow(dead_code)]
pub fn warn<S: AsRef<str>>(message: S) {
    LOGGER.lock().unwrap().log("warn", message.as_ref()).ok();
}

#[allow(dead_code)]
pub fn error<S: AsRef<str>>(message: S) {
    LOGGER.lock().unwrap().log("error", message.as_ref()).ok();
}
