use std::sync::Arc;

use crate::tracer::OtlpTracer;
use crate::metric_base::MetricBase;
use crate::utilities;

pub struct Gauge {
    base: MetricBase,
}

impl Gauge {
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

    pub fn set(&self, value: i64) {
        self.base.set_value(value);
    }

    pub async fn upload(&self) -> simple_error::SimpleResult<()> {
        let current_time = utilities::nanos().to_string();
        self.base.upload_metric(current_time, false, 1).await // 1 = Delta
    }
}
