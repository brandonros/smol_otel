use std::sync::Arc;

use simple_error::SimpleResult;
use smol::MainExecutor as _;
use smol::Executor;

async fn async_main(_executor: &Arc<Executor<'static>>) -> SimpleResult<()> {
    Ok(())
}

fn main() -> SimpleResult<()> {
    Arc::<Executor>::with_main(|executor| smol::block_on(async_main(executor)))
}
