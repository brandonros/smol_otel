use std::{collections::HashMap, sync::Arc};

use rand::Rng as _;
use smol::Executor;

use crate::{structs::*, tracer::OtlpTracer};

impl Span {
    fn generate_trace_id() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
        hex::encode(bytes)
    }

    fn generate_span_id() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 8];
        rng.fill(&mut bytes);
        hex::encode(bytes)
    }
}

pub struct SpanGuard {
    executor: Arc<Executor<'static>>,
    tracer: Arc<OtlpTracer>,
    start_time: u128,
    name: String,
}

impl SpanGuard {
    pub fn start(executor: &Arc<Executor<'static>>, tracer: &Arc<OtlpTracer>, name: &str) -> Self {
        Self {
            executor: executor.clone(),
            tracer: tracer.clone(),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            name: name.to_string(),
        }
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let tracer_clone = self.tracer.clone();
        let executor_clone = self.executor.clone();
        let name_clone = self.name.clone();
        let start_time = self.start_time;
        let handle = executor_clone.spawn(async move {
            // Create your span here using self.start_time and end_time
            let resource = Resource {
                attributes: {
                    let mut map = HashMap::new();
                    map.insert("telemetry.sdk.language".to_string(), "rust".to_string());
                    map.insert("service.name".to_string(), "unknown_service".to_string());
                    map.insert("telemetry.sdk.version".to_string(), "0.26.0".to_string());
                    map.insert("telemetry.sdk.name".to_string(), "opentelemetry".to_string());
                    Attributes::from(map).0
                },
                dropped_attributes_count: 0,
            };
            let scope = Scope {
                name: "chess-bot".to_string(),
                version: "0.1.0".to_string(),
                attributes: vec![],
                dropped_attributes_count: 0,
            };
            let span = Span {
                trace_id: Span::generate_trace_id(),
                span_id: Span::generate_span_id(),
                parent_span_id: "".to_string(), // TODO
                name: name_clone,
                start_time_unix_nano: start_time.to_string(),
                end_time_unix_nano: end_time.to_string(),
                kind: SpanKind::Internal as i64,
                flags: TraceFlags::Sampled as i64,
                trace_state: "".to_string(),
                attributes: {
                    let mut map = HashMap::new();
                    map.insert("code.filepath".to_string(), "src/main.rs".to_string());
                    map.insert("code.namespace".to_string(), "chess_bot".to_string());
                    map.insert("code.lineno".to_string(), "71".to_string());
                    map.insert("thread.id".to_string(), "1".to_string());
                    map.insert("thread.name".to_string(), "main".to_string());
                    Attributes::from(map).0
                },
                events: vec![],
                links: vec![],
                dropped_links_count: 0, // TODO
                dropped_attributes_count: 0, // TODO
                dropped_events_count: 0, // TODO                
                status: Status {
                    message: "".to_string(),
                    code: 0,
                }
            };
            let resource_span = ResourceSpan {
                resource,
                scope_spans: vec![ScopeSpan {
                    scope,
                    spans: vec![span],
                }],
            };
            
            // Handle any errors here since we can't propagate them
            if let Err(e) = tracer_clone.upload_traces(vec![resource_span]).await {
                eprintln!("Failed to upload span: {}", e);
            }
        });
        handle.detach();
    }
}
