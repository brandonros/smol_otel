use std::sync::Arc;

use simple_error::SimpleResult;
use smol::MainExecutor as _;
use smol::Executor;
use smol::Timer;
use smol_traces::OtlpTracer;
use smol_traces::SpanGuard;

async fn do_work3(executor: &Arc<Executor<'static>>, tracer: &Arc<OtlpTracer>) -> SimpleResult<()> {
    let _guard = SpanGuard::start(&executor, &tracer, "do_work3");
    log::info!("hello from do_work3");
    Ok(())
}

async fn do_work2(executor: &Arc<Executor<'static>>, tracer: &Arc<OtlpTracer>) -> SimpleResult<()> {
    let _guard = SpanGuard::start(&executor, &tracer, "do_work2");
    log::info!("hello from do_work2");
    do_work3(executor, tracer).await?;
    Ok(())
}

async fn do_work1(executor: &Arc<Executor<'static>>, tracer: &Arc<OtlpTracer>) -> SimpleResult<()> {
    let _guard = SpanGuard::start(&executor, &tracer, "do_work1");
    log::info!("hello from do_work1");
    do_work2(executor, tracer).await?;
    Ok(())
}

async fn async_main(executor: &Arc<Executor<'static>>) -> SimpleResult<()> {
    // init logger
    smol_traces::logger::init()?;

    // create tracer
    let endpoint = "http://tempo.node.external/v1/traces";
    let service_name = "smol_tracer";
    let tracer = OtlpTracer::new(endpoint, service_name)?;
    let tracer = Arc::new(tracer);

    // create span
    let guard = SpanGuard::start(&executor, &tracer, "async_main");

    // log
    log::info!("hello, world!");

    // do work
    do_work1(&executor, &tracer).await?;

    // drop
    drop(guard);

    // wait forever
    Timer::never().await;

    // return
    Ok(())
}

fn main() -> SimpleResult<()> {
    Arc::<Executor>::with_main(|executor| smol::block_on(async_main(executor)))
}
