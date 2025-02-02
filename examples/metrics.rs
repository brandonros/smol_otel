use std::sync::Arc;
use std::time::Duration;
use simple_error::SimpleResult;
use smol::MainExecutor as _;
use smol::{Executor, Timer};
use smol_otel::{Counter, Gauge, OtlpTracer};

async fn async_main(_executor: Arc<Executor<'static>>) -> SimpleResult<()> {
    // init logger
    smol_otel::logger::init()?;

    // create tracer
    let traces_endpoint = "https://otlp-gateway-prod-us-east-0.grafana.net/otlp/v1/traces";
    let metrics_endpoint = "https://otlp-gateway-prod-us-east-0.grafana.net/otlp/v1/metrics";
    let service_name = "order-processing-service";
    let tracer = OtlpTracer::new(traces_endpoint, metrics_endpoint, service_name)?;
    let tracer = Arc::new(tracer);

    // Create a counter for processed orders
    let orders_processed = Counter::new(
        tracer.clone(),
        "orders_processed",
        "Number of orders processed",
        "orders",
    )
    .with_attribute("customer.id", "customer456")
    .with_attribute("order.status", "success")
    .with_attribute("order.type", "standard");

    // Create a gauge for current queue size
    let queue_size = Gauge::new(
        tracer.clone(),
        "order_queue_size",
        "Current number of orders in queue",
        "orders",
    )
    .with_attribute("queue.type", "standard");

    // Simulate some metrics
    for i in 0..5 {
        // Increment processed orders
        orders_processed.inc();
        orders_processed.upload().await?;
        log::info!("Order processed: {}", i + 1);

        // Update queue size
        queue_size.set(10 - i);
        queue_size.upload().await?;
        log::info!("Queue size updated: {}", 10 - i);

        Timer::after(Duration::from_secs(1)).await;
    }

    log::info!("Metrics example completed!");
    Timer::never().await;

    Ok(())
}

fn main() -> SimpleResult<()> {
    Arc::<Executor>::with_main(|executor| smol::block_on(async_main(executor.clone())))
}
