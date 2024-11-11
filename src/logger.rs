use log::{Log, Metadata, Record};
use simple_error::{box_err, SimpleResult};

use crate::span::CURRENT_SPAN_GUARD;

pub struct SpanLogger;

impl Log for SpanLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Check if the log level is enabled according to the max level filter
        metadata.level() <= log::max_level()
    }

    #[track_caller]
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Print to console
            println!("[{}] {}", record.level(), record.args());

            // push log to current span
            CURRENT_SPAN_GUARD.with(|current| {
                if let Some(guard) = current.borrow().as_ref() {
                    guard.push_event(record.level(), record.args());
                }
            });
        }
    }

    fn flush(&self) {}
}

pub fn init() -> SimpleResult<()> {
    log::set_logger(&SpanLogger)
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
        .map_err(|e| box_err!(format!("failed to set logger: {}", e)))
}
