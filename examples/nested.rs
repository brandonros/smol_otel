use std::sync::Arc;

use simple_error::SimpleResult;
use smol::MainExecutor as _;
use smol::{Executor, Timer};
use smol_otel::{OtlpTracer, SpanKind, StatusCode};

async fn do_work3(executor: Arc<Executor<'static>>, tracer: Arc<OtlpTracer>) -> SimpleResult<()> {
    let guard = tracer
        .span("do_work3")
        .with_attribute("key1", "value1")
        .with_status("error", StatusCode::Error)
        .with_kind(SpanKind::Internal)
        .start(executor.clone());
    log::info!("hello from do_work3");
    guard.set_attribute("key2", "value2");
    Ok(())
}

async fn do_work2(executor: Arc<Executor<'static>>, tracer: Arc<OtlpTracer>) -> SimpleResult<()> {
    let _guard = tracer
        .span("do_work2")
        .with_kind(SpanKind::Internal)
        .start(executor.clone());
    log::info!("hello from do_work2");
    do_work3(executor.clone(), tracer.clone()).await?;
    Ok(())
}

async fn do_work1(executor: Arc<Executor<'static>>, tracer: Arc<OtlpTracer>) -> SimpleResult<()> {
    let _guard = tracer
        .span("do_work1")
        .with_kind(SpanKind::Internal)
        .start(executor.clone());
    log::info!("hello from do_work1");
    do_work2(executor.clone(), tracer.clone()).await?;
    Ok(())
}

async fn async_main(executor: Arc<Executor<'static>>) -> SimpleResult<()> {
    // init logger
    smol_otel::logger::init()?;

    // create tracer
    let endpoint = "http://tempo.node.external/v1/traces";
    let service_name = "smol_tracer";
    let tracer = OtlpTracer::new(endpoint, service_name)?;
    let tracer = Arc::new(tracer);

    // create span
    let guard = tracer
        .span("async_main")
        .with_kind(SpanKind::Internal)
        .start(executor.clone());

    // log
    log::info!("hello, world!");

    // do work
    do_work1(executor.clone(), tracer.clone()).await?;

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
