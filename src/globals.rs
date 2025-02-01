use std::sync::Arc;

use smol::Executor;
use once_cell::sync::OnceCell;

use crate::tracer::OtlpTracer;

static GLOBAL_EXECUTOR: OnceCell<Arc<Executor<'static>>> = OnceCell::new();
static GLOBAL_TRACER: OnceCell<Arc<OtlpTracer>> = OnceCell::new();

pub fn register(executor: Arc<Executor<'static>>, tracer: Arc<OtlpTracer>) {
    GLOBAL_EXECUTOR.set(executor).expect("Failed to set global executor");
    GLOBAL_TRACER.set(tracer).expect("Failed to set global tracer");
}

pub fn executor() -> &'static Arc<Executor<'static>> {
    GLOBAL_EXECUTOR.get().expect("Global executor not initialized")
}

pub fn tracer() -> &'static Arc<OtlpTracer> {
    GLOBAL_TRACER.get().expect("Global tracer not initialized")
}
