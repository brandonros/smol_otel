use std::sync::{Arc, OnceLock};
use smol::Executor;

use crate::tracer::OtlpTracer;

static GLOBAL_EXECUTOR: OnceLock<Arc<Executor<'static>>> = OnceLock::new();
static GLOBAL_TRACER: OnceLock<Arc<OtlpTracer>> = OnceLock::new();

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
