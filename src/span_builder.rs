use std::collections::HashMap;
use std::sync::Arc;

use crate::tracer::OtlpTracer;
use crate::globals;
use crate::structs::*;
use crate::span_guard::SpanGuard;

pub struct SpanBuilder {
    tracer: Arc<OtlpTracer>,
    name: String,
    status: Status,
    kind: SpanKind,
    attributes: HashMap<String, String>,
}

impl SpanBuilder {
    pub(crate) fn new(tracer: Arc<OtlpTracer>, name: &str) -> Self {
        Self {
            tracer,
            name: name.to_string(),
            status: Status { message: "".to_string(), code: StatusCode::Unset as i64 },
            kind: SpanKind::Internal,
            attributes: HashMap::new(),
        }
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(
            key.into(), 
            value.into()
        );
        self
    }

    pub fn with_status(mut self, message: &str, code: StatusCode) -> Self {
        self.status = Status { message: message.to_string(), code: code as i64 };
        self
    }

    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn start(self) -> Arc<SpanGuard> {
        SpanGuard::start(
            &globals::executor(), 
            &self.tracer, 
            &self.name,
            self.status,
            self.kind,
            self.attributes
        )
    }
}