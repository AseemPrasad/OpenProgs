use opentelemetry::trace::{Tracer, Span};
use opentelemetry::global;
use opentelemetry::Key;
use tracing::Instrument;
use std::time::Instant;

pub struct SpanInstrumentation;

impl SpanInstrumentation {
    pub async fn trace_policy_evaluation<F, T>(
        tenant_id: &str,
        policy_name: &str,
        f: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let tracer = global::tracer("secureai-policy");
        let mut span = tracer.start("policy.evaluate");
        span.set_attribute(Key::new("tenant_id").string(tenant_id));
        span.set_attribute(Key::new("policy_name").string(policy_name));

        f.instrument(tracing::debug_span!("policy_eval")).await
    }

    pub async fn trace_sandbox_execution<F, T>(
        tenant_id: &str,
        vm_id: &str,
        task_id: &str,
        f: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let tracer = global::tracer("secureai-sandbox");
        let mut span = tracer.start("sandbox.execute");
        span.set_attribute(Key::new("tenant_id").string(tenant_id));
        span.set_attribute(Key::new("vm_id").string(vm_id));
        span.set_attribute(Key::new("task_id").string(task_id));

        f.instrument(tracing::debug_span!("sandbox_exec")).await
    }

    pub async fn trace_stream_proxy<F, T>(
        tenant_id: &str,
        provider: &str,
        model: &str,
        f: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let tracer = global::tracer("secureai-proxy");
        let mut span = tracer.start("proxy.stream");
        span.set_attribute(Key::new("tenant_id").string(tenant_id));
        span.set_attribute(Key::new("provider").string(provider));
        span.set_attribute(Key::new("model").string(model));

        f.instrument(tracing::debug_span!("stream_proxy")).await
    }

    pub async fn trace_guardrail_check<F, T>(
        tenant_id: &str,
        threat_type: &str,
        f: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let tracer = global::tracer("secureai-guardrails");
        let mut span = tracer.start("guardrail.check");
        span.set_attribute(Key::new("tenant_id").string(tenant_id));
        span.set_attribute(Key::new("threat_type").string(threat_type));

        f.instrument(tracing::debug_span!("guardrail_check")).await
    }

    pub async fn trace_circuit_breaker<F, T>(
        provider_name: &str,
        f: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let tracer = global::tracer("secureai-router");
        let mut span = tracer.start("router.circuit_breaker");
        span.set_attribute(Key::new("provider_name").string(provider_name));

        f.instrument(tracing::debug_span!("circuit_breaker")).await
    }

    pub fn record_span_duration(span: &mut dyn Span, duration_ms: u64) {
        span.set_attribute(Key::new("duration_ms").i64(duration_ms as i64));
    }

    pub fn record_span_status(span: &mut dyn Span, status: &str) {
        span.set_attribute(Key::new("status").string(status));
    }

    pub fn record_span_error(span: &mut dyn Span, error_msg: &str) {
        span.set_attribute(Key::new("error").bool(true));
        span.set_attribute(Key::new("error_message").string(error_msg));
    }
}

pub struct ScopedSpan {
    start_time: Instant,
    span_name: String,
}

impl ScopedSpan {
    pub fn new(span_name: impl Into<String>) -> Self {
        let name = span_name.into();
        let tracer = global::tracer("secureai-default");
        let _span = tracer.start(&name);

        Self {
            start_time: Instant::now(),
            span_name: name,
        }
    }

    pub fn duration_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

impl Drop for ScopedSpan {
    fn drop(&mut self) {
        let duration_ms = self.duration_ms();
        tracing::debug!("{} took {}ms", self.span_name, duration_ms);
    }
}
