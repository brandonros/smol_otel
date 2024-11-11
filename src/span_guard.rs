use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as SyncMutex;
use smol::Executor;

use crate::span_context::SpanContext;
use crate::structs::*;
use crate::tracer::OtlpTracer;
use crate::utilities;

thread_local! {
    pub static CURRENT_SPAN_CONTEXT: RefCell<Option<SpanContext>> = RefCell::new(None);
    pub static CURRENT_SPAN_GUARD: RefCell<Option<Arc<SpanGuard>>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct SpanGuard {
    executor: Arc<Executor<'static>>,
    tracer: Arc<OtlpTracer>,
    start_time: u128,
    name: String,
    file: String,
    line: u32,
    column: u32,
    context: SpanContext,
    kind: SpanKind,
    parent_context: Option<SpanContext>,
    thread_id: String,
    thread_name: String,
    events: Arc<SyncMutex<Vec<Event>>>,
    attributes: Arc<SyncMutex<HashMap<String, String>>>,
    status: Arc<SyncMutex<Status>>,
}

impl SpanGuard {
    #[track_caller]
    pub fn start(
        executor: &Arc<Executor<'static>>, 
        tracer: &Arc<OtlpTracer>, 
        name: &str,
        status: Status,
        kind: SpanKind,
        attributes: HashMap<String, String>
    ) -> Arc<Self> {
        let location = std::panic::Location::caller();

        // Capture the parent context before creating new span
        let parent_context = CURRENT_SPAN_CONTEXT.with(|current| {
            current.borrow().clone()
        });
        
        // Generate new span context
        let trace_id = if let Some(parent) = &parent_context {
            parent.trace_id.clone() // Inherit trace_id from parent
        } else {
            utilities::generate_trace_id()
        };
        let span_id = utilities::generate_span_id();
        let new_context = SpanContext {
            trace_id,       
            span_id
        };
        
        // Set as current context
        CURRENT_SPAN_CONTEXT.with(|current| {
            *current.borrow_mut() = Some(new_context.clone());
        });
        
        // Get thread info at span start
        let thread = std::thread::current();
        let thread_id = format!("{:?}", thread.id());
        let thread_name = thread.name().unwrap_or("unnamed").to_string();
        
        // Build guard
        let guard = Self {
            executor: executor.clone(),
            tracer: tracer.clone(),
            start_time: utilities::nanos(),
            name: name.to_string(),
            file: location.file().to_string(),
            line: location.line(),
            column: location.column(),
            context: new_context,
            kind,
            parent_context,
            thread_id,
            thread_name,
            events: Arc::new(SyncMutex::new(vec![])),
            attributes: Arc::new(SyncMutex::new(attributes)),
            status: Arc::new(SyncMutex::new(status)),
        };

        // Wrap the guard in an Arc
        let guard = Arc::new(guard);

        // Store Arc in thread local
        CURRENT_SPAN_GUARD.with(|current| {
            *current.borrow_mut() = Some(guard.clone());
        });

        guard
    }

    pub fn push_event(&self, level: log::Level, args: &std::fmt::Arguments) {
        let time = utilities::nanos();
        
        let event = Event {
            name: args.to_string(),
            time_unix_nano: time.to_string(),
            attributes: {
                let mut map = HashMap::new();
                map.insert("log.level".to_string(), level.to_string());
                Attributes::from(map).0
            },
        };

        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    pub fn set_attribute(&self, key: &str, value: &str) {
        if let Ok(mut attributes) = self.attributes.lock() {
            attributes.insert(key.to_string(), value.to_string());
        }
    }

    pub fn set_status(&self, message: &str, code: StatusCode) {
        if let Ok(mut status) = self.status.lock() {
            status.message = message.to_string();
            status.code = code as i64;
        }
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {        
        // Restore parent context
        if let Some(parent_ctx) = &self.parent_context {
            CURRENT_SPAN_CONTEXT.with(|current| {
                *current.borrow_mut() = Some(parent_ctx.clone());
            });
        } else {
            // Only clear if no parent context
            CURRENT_SPAN_GUARD.with(|current| {
                *current.borrow_mut() = None;
            });
        }

        // Get end time
        let end_time = utilities::nanos();
        
        // Prepare data for span
        let status = self.status.lock().unwrap().clone();
        let span_attributes = self.attributes.lock().unwrap().clone();
        let events = self.events.lock().unwrap().clone();
        let resource = Resource {
            attributes: {
                let mut map = HashMap::new();
                map.insert("telemetry.sdk.language".to_string(), "rust".to_string());
                map.insert("telemetry.sdk.version".to_string(), "0.1.0".to_string());
                map.insert("telemetry.sdk.name".to_string(), "opentelemetry".to_string());
                map.insert("service.name".to_string(), self.tracer.service_name.clone());
                Attributes::from(map).0
            },
            dropped_attributes_count: 0,
        };
        let scope = Scope {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            attributes: vec![],
            dropped_attributes_count: 0,
        };
        let span = Span {
            trace_id: self.context.trace_id.clone(),
            span_id: self.context.span_id.clone(),
            parent_span_id: self.parent_context.as_ref()
                .map(|ctx| ctx.span_id.clone())
                .unwrap_or_else(|| "".to_string()),
            name: self.name.clone(),
            start_time_unix_nano: self.start_time.to_string(),
            end_time_unix_nano: end_time.to_string(),
            kind: self.kind.clone() as i64,
            flags: TraceFlags::Sampled as i64,
            trace_state: "".to_string(),
            attributes: {
                let mut map = HashMap::new();
                map.insert("code.filepath".to_string(), self.file.clone());
                map.insert("code.lineno".to_string(), self.line.to_string());
                map.insert("code.column".to_string(), self.column.to_string());
                map.insert("thread.id".to_string(), self.thread_id.clone());
                map.insert("thread.name".to_string(), self.thread_name.clone());
                for (key, value) in span_attributes.iter() {
                    map.insert(key.clone(), value.clone());
                }
                Attributes::from(map).0
            },
            events,
            links: vec![],
            dropped_links_count: 0, // TODO
            dropped_attributes_count: 0, // TODO
            dropped_events_count: 0, // TODO                
            status
        };
        let resource_span = ResourceSpan {
            resource,
            scope_spans: vec![ScopeSpan {
                scope,
                spans: vec![span],
            }],
        };

        // Upload span
        let tracer_clone = self.tracer.clone();
        let executor_clone = self.executor.clone();
        let handle = executor_clone.spawn(async move {
            // Handle any errors here since we can't propagate them
            if let Err(e) = tracer_clone.upload_traces(vec![resource_span]).await {
                eprintln!("Failed to upload span: {}", e);
            }
        });
        handle.detach();
    }
}
