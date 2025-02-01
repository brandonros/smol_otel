use std::sync::Arc;
use simple_error::SimpleResult;
use smol::MainExecutor as _;
use smol::{Executor, Timer};
use smol_otel::{Attribute, AttributeValue, DataPoint, Metric, OtlpTracer, Resource, ResourceMetrics, Scope, ScopeMetrics, Sum};

async fn async_main(_executor: Arc<Executor<'static>>) -> SimpleResult<()> {
    // init logger
    smol_otel::logger::init()?;

    // create tracer
    let traces_endpoint = "http://localhost:4318/v1/traces";
    let metrics_endpoint = "http://localhost:4318/v1/metrics";
    let service_name = "order-processing-service";
    let tracer = OtlpTracer::new(traces_endpoint, metrics_endpoint, service_name)?;
    let tracer = Arc::new(tracer);

    // Create metric data
    let data_point = DataPoint {
        attributes: vec![
            Attribute {
                key: "customer.id".to_string(),
                value: AttributeValue {
                    string_value: "customer456".to_string(),
                },
            },
            Attribute {
                key: "order.status".to_string(),
                value: AttributeValue {
                    string_value: "success".to_string(),
                },
            },
            Attribute {
                key: "order.type".to_string(),
                value: AttributeValue {
                    string_value: "standard".to_string(),
                },
            },
        ],
        start_time_unix_nano: "1738431463016000000".to_string(),
        time_unix_nano: "1738431472925000000".to_string(),
        as_double: 1,
    };

    let metric = Metric {
        name: "orders_processed".to_string(),
        description: "Number of orders processed".to_string(),
        unit: "orders".to_string(),
        sum: Sum {
            aggregation_temporality: 2,
            is_monotonic: true,
            data_points: vec![data_point],
        },
    };

    let scope_metrics = ScopeMetrics {
        scope: Scope {
            name: "order-processing-instrumentation".to_string(),
            version: "".to_string(),
            attributes: vec![],
            dropped_attributes_count: 0,
        },
        metrics: vec![metric],
    };

    let resource_metrics = ResourceMetrics {
        resource: Resource {
            attributes: vec![
                Attribute {
                    key: "service.name".to_string(),
                    value: AttributeValue {
                        string_value: service_name.to_string(),
                    },
                },
                Attribute {
                    key: "telemetry.sdk.language".to_string(),
                    value: AttributeValue {
                        string_value: "rust".to_string(),
                    },
                },
                Attribute {
                    key: "telemetry.sdk.name".to_string(),
                    value: AttributeValue {
                        string_value: "opentelemetry".to_string(),
                    },
                },
                Attribute {
                    key: "telemetry.sdk.version".to_string(),
                    value: AttributeValue {
                        string_value: "1.0.0".to_string(),
                    },
                },
                Attribute {
                    key: "service.version".to_string(),
                    value: AttributeValue {
                        string_value: "1.0.0".to_string(),
                    },
                },
            ],
            dropped_attributes_count: 0,
        },
        scope_metrics: vec![scope_metrics],
    };

    // Upload metrics
    log::info!("Uploading metrics...");
    tracer.upload_metrics(vec![resource_metrics]).await?;
    log::info!("Metrics uploaded successfully!");

    // wait forever
    Timer::never().await;

    Ok(())
}

fn main() -> SimpleResult<()> {
    Arc::<Executor>::with_main(|executor| smol::block_on(async_main(executor.clone())))
}
