use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::collections::HashMap;

use crate::structs::*;
use crate::utilities;
use crate::tracer::OtlpTracer;

pub(crate) struct MetricBase {
    tracer: Arc<OtlpTracer>,
    name: String,
    description: String,
    unit: String,
    value: Arc<AtomicI64>,
    attributes: HashMap<String, String>,
}

impl MetricBase {
    pub fn new(
        tracer: Arc<OtlpTracer>,
        name: &str,
        description: &str,
        unit: &str,
    ) -> Self {
        Self {
            tracer,
            name: name.to_string(),
            description: description.to_string(),
            unit: unit.to_string(),
            value: Arc::new(AtomicI64::new(0)),
            attributes: HashMap::new(),
        }
    }

    pub fn with_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn get_value(&self) -> i64 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn set_value(&self, value: i64) {
        self.value.store(value, Ordering::SeqCst);
    }

    pub fn add_value(&self, value: i64) {
        self.value.fetch_add(value, Ordering::SeqCst);
    }

    pub async fn upload_metric(&self, start_time: String, is_monotonic: bool, aggregation_temporality: i64) -> simple_error::SimpleResult<()> {
        let data_point = DataPoint {
            attributes: Attributes::from(self.attributes.clone()).0,
            start_time_unix_nano: start_time,
            time_unix_nano: utilities::nanos().to_string(),
            as_double: self.get_value(),
        };

        let metric = Metric {
            name: self.name.clone(),
            description: self.description.clone(),
            unit: self.unit.clone(),
            sum: Sum {
                aggregation_temporality,
                is_monotonic,
                data_points: vec![data_point],
            },
        };

        let scope_metrics = ScopeMetrics {
            scope: Scope {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                attributes: vec![],
                dropped_attributes_count: 0,
            },
            metrics: vec![metric],
        };

        let resource_metrics = ResourceMetrics {
            resource: Resource {
                attributes: {
                    let mut map = HashMap::new();
                    map.insert("service.name".to_string(), self.tracer.service_name.clone());
                    map.insert("telemetry.sdk.language".to_string(), "rust".to_string());
                    map.insert("telemetry.sdk.name".to_string(), "opentelemetry".to_string());
                    map.insert("telemetry.sdk.version".to_string(), "1.0.0".to_string());
                    Attributes::from(map).0
                },
                dropped_attributes_count: 0,
            },
            scope_metrics: vec![scope_metrics],
        };

        self.tracer.upload_metrics(vec![resource_metrics]).await
    }
} 