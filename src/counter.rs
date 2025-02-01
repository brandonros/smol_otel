use std::sync::Arc;
use std::time::SystemTime;

use crate::tracer::OtlpTracer;
use crate::metric_base::MetricBase;

pub struct Counter {
    base: MetricBase,
}

impl Counter {
    pub fn new(
        tracer: Arc<OtlpTracer>,
        name: &str,
        description: &str,
        unit: &str,
    ) -> Self {
        Self {
            base: MetricBase::new(tracer, name, description, unit),
        }
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.base.with_attribute(key, value);
        self
    }

    pub fn inc(&self) {
        self.add(1);
    }

    pub fn add(&self, value: i64) {
        self.base.add_value(value);
    }

    pub async fn upload(&self) -> simple_error::SimpleResult<()> {
        let start_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();

        self.base.upload_metric(start_time, true, 2).await // 2 = Cumulative
    }
}