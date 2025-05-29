use log::{LevelFilter, Metadata, Record};

pub fn init(log_level: Option<&str>) {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    let level = match log_level {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Info, // 默认为 Info 级别
    };
    // FIXME: Configure the logger
    log::set_max_level(level);

    info!("Logger Initialized.");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        // FIXME: Implement the logger with serial output
        let RED = "[0;31m";
        let YELLOW = "[0;33m";
        let BLUE = "[0;34m";
        let GREEN = "[0;32m";
        let _CYAN = "[0;36m";
        if self.enabled(record.metadata()) {
            match record.level() {
                log::Level::Error => {
                    print!("{}[ERROR]{} {}\n\r", RED, "[0m", record.args());
                }
                log::Level::Warn => {
                    print!("{}[WARN]{} {}\n\r", YELLOW, "[0m", record.args());
                }
                log::Level::Info => {
                    print!("{}[INFO]{} {}\n\r", BLUE, "[0m", record.args());
                }
                log::Level::Debug => {
                    print!("{}[DEBUG]{} {}\n\r", GREEN, "[0m", record.args());
                }
                log::Level::Trace => {
                    print!("{}[TRACE]{} {}\n\r", _CYAN, "[0m", record.args());
                }
            }
        }
    }

    fn flush(&self) {}
}
