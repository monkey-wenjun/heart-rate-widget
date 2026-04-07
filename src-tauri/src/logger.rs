use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use std::path::PathBuf;
use once_cell::sync::Lazy;

static LOGGER: Lazy<Mutex<Logger>> = Lazy::new(|| {
    Mutex::new(Logger::new())
});

pub struct Logger {
    file: Option<File>,
}

impl Logger {
    fn new() -> Self {
        Self { file: None }
    }

    fn init(&mut self) {
        if self.file.is_none() {
            if let Some(config_dir) = dirs::config_dir() {
                let log_dir = config_dir.join("heart-rate-widget");
                let _ = std::fs::create_dir_all(&log_dir);
                let log_file = log_dir.join("app.log");
                
                self.file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file)
                    .ok();
            }
        }
    }

    fn log(&mut self, message: &str) {
        self.init();
        if let Some(file) = &mut self.file {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let _ = writeln!(file, "[{}] {}", timestamp, message);
            let _ = file.flush();
        }
    }
}

pub fn log(message: &str) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log(message);
    }
}

#[macro_export]
macro_rules! app_log {
    ($($arg:tt)*) => {
        $crate::logger::log(&format!($($arg)*))
    };
}
