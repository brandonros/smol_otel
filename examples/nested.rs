use std::sync::Arc;

use simple_error::SimpleResult;
use smol::MainExecutor as _;
use smol::{Executor, Timer};
use smol_otel::{globals, OtlpTracer, SpanKind, StatusCode};

async fn do_work3() -> SimpleResult<()> {
    let guard = globals::tracer()
        .span("do_work3")
        .with_attribute("key1", "value1")
        .with_status("error", StatusCode::Error)
        .start();
    log::info!("hello from do_work3");
    guard.set_attribute("key2", "value2");
    Ok(())
}

async fn do_work2() -> SimpleResult<()> {
    let _guard = globals::tracer()
        .span("do_work2")
        .start();
    log::info!("hello from do_work2");
    do_work3().await?;
    Ok(())
}

async fn do_work1() -> SimpleResult<()> {
    let _guard = globals::tracer()
        .span("do_work1")
        .with_kind(SpanKind::Internal)
        .start();
    log::info!("hello from do_work1");
    do_work2().await?;
    Ok(())
}

async fn async_main(executor: Arc<Executor<'static>>) -> SimpleResult<()> {
    // init logger
    smol_otel::logger::init()?;

    // create tracer
    let traces_endpoint = "http://tempo.node.external/v1/traces";
    let metrics_endpoint = "http://tempo.node.external/v1/metrics";
    let service_name = "smol_tracer";
    let tracer = OtlpTracer::new(traces_endpoint, metrics_endpoint, service_name)?;
    let tracer = Arc::new(tracer);

    // register globals
    smol_otel::globals::register(executor.clone(), tracer.clone());

    // create span
    let guard = globals::tracer()
        .span("async_main")
        .with_kind(SpanKind::Internal)
        .start();

    // log
    log::info!("hello, world!");

    // do work
    do_work1().await?;

    // drop
    drop(guard);

    // wait forever
    Timer::never().await;

    // return
    Ok(())
}

fn main() -> SimpleResult<()> {
    Arc::<Executor>::with_main(|executor| smol::block_on(async_main(executor.clone())))
}
