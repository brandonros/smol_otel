use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::collections::HashMap;

use crate::structs::*;
use crate::utilities;
use crate::tracer::OtlpTracer;

pub struct Gauge {
    tracer: Arc<OtlpTracer>,
    name: String,
    description: String,
    unit: String,
    value: Arc<AtomicI64>,
    attributes: HashMap<String, String>,
}

impl Gauge {
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

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    pub fn set(&self, value: i64) {
        self.value.store(value, Ordering::SeqCst);
    }

    pub async fn upload(&self) -> simple_error::SimpleResult<()> {
        let data_point = DataPoint {
            attributes: Attributes::from(self.attributes.clone()).0,
            start_time_unix_nano: utilities::nanos().to_string(),
            time_unix_nano: utilities::nanos().to_string(),
            as_double: self.value.load(Ordering::SeqCst),
        };

        let metric = Metric {
            name: self.name.clone(),
            description: self.description.clone(),
            unit: self.unit.clone(),
            sum: Sum {
                aggregation_temporality: 1, // Delta
                is_monotonic: false,
                data_points: vec![data_point],
            },
        };

        self.upload_metric(metric).await
    }

    async fn upload_metric(&self, metric: Metric) -> simple_error::SimpleResult<()> {
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
