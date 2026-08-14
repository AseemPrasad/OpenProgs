use std::sync::Arc;
use parking_lot::RwLock;
use serde_json::json;

pub type AuditLedgerRef = Arc<RwLock<crate::audit::AuditLedger>>;

pub struct AuditHooks {
    ledger: Option<AuditLedgerRef>,
}

impl AuditHooks {
    pub fn new(ledger: Option<AuditLedgerRef>) -> Self {
        Self { ledger }
    }

    pub fn log_policy_validation(
        &self,
        tenant_id: &str,
        model: &str,
        allowed: bool,
    ) {
        if let Some(ref ledger) = self.ledger {
            let details = json!({
                "model": model,
                "allowed": allowed
            });

            let mut ledger_guard = ledger.write();
            let _ = ledger_guard.append_entry("policy.validate_task", tenant_id, details);
        }
    }

    pub fn log_sandbox_execution(
        &self,
        tenant_id: &str,
        vm_id: &str,
        status: &str,
        duration_ms: u64,
    )
    {
        if let Some(ref ledger) = self.ledger {
            let details = json!({
                "vm_id": vm_id,
                "status": status,
                "duration_ms": duration_ms
            });

            let mut ledger_guard = ledger.write();
            let _ = ledger_guard.append_entry("sandbox.execute", tenant_id, details);
        }
    }

    pub fn log_stream_proxy_event(
        &self,
        tenant_id: &str,
        provider: &str,
        model: &str,
        tokens_consumed: u32,
        status: &str,
    )
    {
        if let Some(ref ledger) = self.ledger {
            let details = json!({
                "provider": provider,
                "model": model,
                "tokens_consumed": tokens_consumed,
                "status": status
            });

            let mut ledger_guard = ledger.write();
            let _ = ledger_guard.append_entry("proxy.stream", tenant_id, details);
        }
    }

    pub fn log_guardrail_trigger(
        &self,
        tenant_id: &str,
        threat_type: &str,
        score: f32,
        decision: &str,
    )
    {
        if let Some(ref ledger) = self.ledger {
            let details = json!({
                "threat_type": threat_type,
                "score": score,
                "decision": decision
            });

            let mut ledger_guard = ledger.write();
            let _ = ledger_guard.append_entry("guardrail.check", tenant_id, details);
        }
    }

    pub fn log_circuit_breaker_event(
        &self,
        tenant_id: &str,
        provider: &str,
        state: &str,
        reason: &str,
    )
    {
        if let Some(ref ledger) = self.ledger {
            let details = json!({
                "provider": provider,
                "state": state,
                "reason": reason
            });

            let mut ledger_guard = ledger.write();
            let _ = ledger_guard.append_entry("router.circuit_breaker", tenant_id, details);
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.ledger.is_some()
    }
}

pub struct GlobalAuditHooks;

static AUDIT_HOOKS: parking_lot::RwLock<Option<AuditHooks>> = parking_lot::RwLock::new(None);

impl GlobalAuditHooks {
    pub fn initialize(ledger: Option<AuditLedgerRef>) {
        let hooks = if ledger.is_some() {
            Some(AuditHooks::new(ledger))
        } else {
            None
        };

        *AUDIT_HOOKS.write() = hooks;
    }

    pub fn log_policy_validation(tenant_id: &str, model: &str, allowed: bool) {
        if let Some(ref hooks) = *AUDIT_HOOKS.read() {
            hooks.log_policy_validation(tenant_id, model, allowed);
        }
    }

    pub fn log_sandbox_execution(
        tenant_id: &str,
        vm_id: &str,
        status: &str,
        duration_ms: u64,
    ) {
        if let Some(ref hooks) = *AUDIT_HOOKS.read() {
            hooks.log_sandbox_execution(tenant_id, vm_id, status, duration_ms);
        }
    }

    pub fn log_stream_proxy_event(
        tenant_id: &str,
        provider: &str,
        model: &str,
        tokens_consumed: u32,
        status: &str,
    ) {
        if let Some(ref hooks) = *AUDIT_HOOKS.read() {
            hooks.log_stream_proxy_event(tenant_id, provider, model, tokens_consumed, status);
        }
    }

    pub fn log_guardrail_trigger(
        tenant_id: &str,
        threat_type: &str,
        score: f32,
        decision: &str,
    ) {
        if let Some(ref hooks) = *AUDIT_HOOKS.read() {
            hooks.log_guardrail_trigger(tenant_id, threat_type, score, decision);
        }
    }

    pub fn log_circuit_breaker_event(
        tenant_id: &str,
        provider: &str,
        state: &str,
        reason: &str,
    ) {
        if let Some(ref hooks) = *AUDIT_HOOKS.read() {
            hooks.log_circuit_breaker_event(tenant_id, provider, state, reason);
        }
    }
}
