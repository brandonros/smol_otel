use std::collections::HashMap;

use miniserde::Serialize;

#[allow(dead_code)]
#[derive(Serialize)]
#[repr(i64)]
pub enum SpanKind {
    Unspecified = 0,  // Default
    Internal = 1,     // Internal operations within an application
    Server = 2,       // Server-side handling of a request
    Client = 3,       // Client-side of a request
    Producer = 4,     // Initiator of an asynchronous request
    Consumer = 5,     // Handler of an asynchronous request
}

#[derive(Serialize)]
#[repr(i64)]
pub enum TraceFlags {
    Sampled = 1
}

#[derive(Serialize)]
pub struct ResourceSpansRoot {
    #[serde(rename = "resourceSpans")]
    pub resource_spans: Vec<ResourceSpan>,
}

#[derive(Serialize)]
pub struct ResourceSpan {
    pub resource: Resource,
    #[serde(rename = "scopeSpans")]
    pub scope_spans: Vec<ScopeSpan>,
}

#[derive(Serialize)]
pub struct Resource {
    pub attributes: Vec<Attribute>,
    #[serde(rename = "droppedAttributesCount")]
    pub dropped_attributes_count: i64,
}

#[derive(Serialize)]
pub struct Attribute {
    pub key: String,
    pub value: AttributeValue,
}

#[derive(Serialize)]
pub struct AttributeValue {
    #[serde(rename = "stringValue")]
    pub string_value: String,
}

#[derive(Serialize)]
pub struct ScopeSpan {
    pub scope: Scope,
    pub spans: Vec<Span>,
}

#[derive(Serialize)]
pub struct Scope {
    pub name: String,
    pub version: String,
    pub attributes: Vec<Attribute>,
    #[serde(rename = "droppedAttributesCount")]
    pub dropped_attributes_count: i64,
}

#[derive(Serialize)]
pub struct Span {
    #[serde(rename = "traceId")]
    pub trace_id: String,
    #[serde(rename = "spanId")]
    pub span_id: String,
    #[serde(rename = "parentSpanId")]
    pub parent_span_id: String,
    pub name: String,
    #[serde(rename = "startTimeUnixNano")]
    pub start_time_unix_nano: String,
    #[serde(rename = "endTimeUnixNano")]
    pub end_time_unix_nano: String,
    pub kind: i64, // SpanKind as int
    pub attributes: Vec<Attribute>,
    pub events: Vec<Event>,
    #[serde(rename = "traceState")]
    pub trace_state: String,
    pub flags: i64,
    #[serde(rename = "droppedAttributesCount")]
    pub dropped_attributes_count: i64,
    #[serde(rename = "droppedEventsCount")]
    pub dropped_events_count: i64,
    pub links: Vec<Link>,
    #[serde(rename = "droppedLinksCount")]
    pub dropped_links_count: i64,
    pub status: Status,
}

#[derive(Serialize)]
pub struct Event {
    pub name: String,
    #[serde(rename = "timeUnixNano")]
    pub time_unix_nano: String,
    pub attributes: Vec<Attribute>,
}

#[derive(Serialize)]
pub struct Link {
    // Add fields if needed
}

#[derive(Serialize)]
pub struct Status {
    pub message: String,
    pub code: i64,
}

pub struct Attributes(pub Vec<Attribute>);

impl From<HashMap<String, String>> for Attributes {
    fn from(map: HashMap<String, String>) -> Self {
        Attributes(
            map.into_iter()
                .map(|(key, string_value)| Attribute {
                    key,
                    value: AttributeValue { string_value },
                })
                .collect()
        )
    }
}
