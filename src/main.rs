mod tracer;
mod structs;
mod span;

use std::sync::Arc;

use simple_error::SimpleResult;
use smol::MainExecutor as _;
use smol::Executor;
use smol::Timer;
use span::SpanGuard;
use tracer::OtlpTracer;

async fn do_work2(executor: &Arc<Executor<'static>>, tracer: &Arc<OtlpTracer>) -> SimpleResult<()> {
    let _guard = SpanGuard::start(&executor, &tracer, "do_work2");
    Ok(())
}

async fn do_work1(executor: &Arc<Executor<'static>>, tracer: &Arc<OtlpTracer>) -> SimpleResult<()> {
    let _guard = SpanGuard::start(&executor, &tracer, "do_work1");
    do_work2(executor, tracer).await?;
    Ok(())
}

async fn async_main(executor: &Arc<Executor<'static>>) -> SimpleResult<()> {
    let endpoint = "http://tempo.node.external/v1/traces";
    let service_name = "smol_tracer";
    let tracer = tracer::OtlpTracer::new(endpoint, service_name)?;
    let tracer = Arc::new(tracer);
    do_work1(&executor, &tracer).await?;
    Timer::never().await;
    Ok(())
}

fn main() -> SimpleResult<()> {
    Arc::<Executor>::with_main(|executor| smol::block_on(async_main(executor)))
}
