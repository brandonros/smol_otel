use std::env;
use std::str::FromStr;

use log::{Log, Metadata, Record};
use miniserde::Serialize;
use simple_error::{box_err, SimpleResult};

use crate::span_guard::CURRENT_SPAN_GUARD;
use crate::utilities;

#[derive(Serialize)]
struct LogMessage {
    timestamp: String,
    level: String,
    module: String,
    message: String,
}

#[derive(Clone, Debug)]
struct LogDirective {
    module: Option<String>,
    level: log::LevelFilter,
}

pub struct SpanLogger {
    directives: Vec<LogDirective>,
}

impl SpanLogger {
    fn new() -> Self {
        let directives = Self::parse_env_directives();
        Self { directives }
    }

    fn parse_env_directives() -> Vec<LogDirective> {
        let mut directives = Vec::new();
        
        // Get RUST_LOG environment variable
        if let Ok(env_filter) = env::var("RUST_LOG") {
            for directive in env_filter.split(',') {
                if directive.is_empty() {
                    continue;
                }
                
                // Parse module=level or just level
                let parts: Vec<&str> = directive.split('=').collect();
                match parts.as_slice() {
                    [module, level] => {
                        if let Ok(level) = log::LevelFilter::from_str(level) {
                            directives.push(LogDirective {
                                module: Some(module.to_string()),
                                level,
                            });
                        }
                    }
                    [level] => {
                        if let Ok(level) = log::LevelFilter::from_str(level) {
                            directives.push(LogDirective {
                                module: None,
                                level,
                            });
                        }
                    }
                    _ => continue,
                }
            }
        }
        
        // Only default to Info if no RUST_LOG was set
        if directives.is_empty() {
            directives.push(LogDirective {
                module: None,
                level: log::LevelFilter::Info,
            });
        }
        
        directives
    }
}

impl Log for SpanLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {        
        // First check module-specific directives
        for directive in &self.directives {
            match &directive.module {
                Some(module) => {
                    if metadata.target().starts_with(module) {
                        return metadata.level() <= directive.level;
                    }
                }
                None => continue  // Skip global directive for now
            }
        }

        // If no module matches, fall back to global directive
        for directive in &self.directives {
            if directive.module.is_none() {
                return metadata.level() <= directive.level;
            }
        }

        false
    }

    #[track_caller]
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Print to console
            let log_message = LogMessage {
                timestamp: utilities::iso_timestamp(),
                level: record.level().to_string().to_lowercase(),
                module: record.target().to_string(),
                message: record.args().to_string(),
            };
            println!("{}", miniserde::json::to_string(&log_message));

            // push log to current span
            CURRENT_SPAN_GUARD.with(|current| {
                if let Some(guard) = current.borrow().as_ref() {
                    guard.push_event(record.level(), record.args());
                }
            });
        }
    }

    fn flush(&self) {

    }
}

pub fn init() -> SimpleResult<()> {
    let logger = SpanLogger::new();
    log::set_logger(Box::leak(Box::new(logger)))
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .map_err(|e| box_err!(format!("failed to set logger: {}", e)))
}
