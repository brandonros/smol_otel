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

async fn do_work(executor: &Arc<Executor<'static>>, tracer: &Arc<OtlpTracer>) -> SimpleResult<()> {
    let _guard = SpanGuard::start(&executor, &tracer, "do_work2");
    Ok(())
}

async fn async_main(executor: &Arc<Executor<'static>>) -> SimpleResult<()> {
    let tracer = tracer::OtlpTracer::new();
    let tracer = Arc::new(tracer);
    do_work(&executor, &tracer).await?;
    Timer::never().await;
    Ok(())
}

fn main() -> SimpleResult<()> {
    Arc::<Executor>::with_main(|executor| smol::block_on(async_main(executor)))
}
