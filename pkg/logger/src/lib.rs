use std::fmt;
use std::sync::Once;
use chrono;
use colored::{self, Colorize};

static INIT: Once = Once::new();
static mut GLOBAL_MIN_LEVEL: Level = Level::Info;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Level {    
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Level::Debug => write!(f, "DEBUG"),
            Level::Info => write!(f, "INFO"), 
            Level::Warn => write!(f, "WARN"),
            Level::Error => write!(f, "ERROR"),
        }
    }  
}

pub fn set_level(level: Level) {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_MIN_LEVEL = level;
        });
    }
}

fn should_log(level: Level) -> bool {
    unsafe { level >= GLOBAL_MIN_LEVEL }
}

pub fn format(level: Level, msg: &str) -> String {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    match level {
        Level::Debug => format!("[DEBUG] {} : {}", timestamp, msg),
        Level::Info => format!("[INFO] {} : {}", timestamp, msg).green().to_string(),
        Level::Warn => format!("[WARN] {} : {}", timestamp, msg).yellow().to_string(),
        Level::Error => format!("[ERROR] {} : {}", timestamp, msg).red().to_string(),
    }
}

fn log(level: Level, msg: &str) {
    if !should_log(level) {
        return;
    }

    println!("{}", format(level, msg));
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::info(&format!($($arg)*))
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::warn(&format!($($arg)*))
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::error(&format!($($arg)*))
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::debug(&format!($($arg)*))
    }
}

pub fn info(msg: &str) {
    log(Level::Info, msg);
}

pub fn warn(msg: &str) {
    log(Level::Warn, msg);
}

pub fn error(msg: &str) {
    log(Level::Error, msg);
}

pub fn debug(msg: &str) {
    log(Level::Debug, msg);
}