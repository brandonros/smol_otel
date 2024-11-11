use log::{Log, Metadata, Record};
use miniserde::Serialize;
use simple_error::{box_err, SimpleResult};

use crate::span_guard::CURRENT_SPAN_GUARD;
use crate::utilities;

#[derive(Serialize)]
struct LogMessage {
    timestamp: String,
    level: String,
    message: String,
}

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
            let log_message = LogMessage {
                timestamp: utilities::iso_timestamp(),
                level: record.level().to_string().to_lowercase(),
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
    // TODO: will this work
    // RUST_LOG=debug,rustls=info,http_client=info,websocket_client=info,thinkorswim::json_patch_state=info,thinkorswim::state=info,thinkorswim::client=info
    // TODO: if not, how to make it work?
    log::set_logger(&SpanLogger)
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
        .map_err(|e| box_err!(format!("failed to set logger: {}", e)))
}
