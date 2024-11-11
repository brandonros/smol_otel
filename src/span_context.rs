#[derive(Clone)]
pub struct SpanContext {
    pub trace_id: String,
    pub span_id: String,
}
