use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct BudgetExhausted {
    pub needed: u32,
    pub available: u32,
}

pub struct TokenBudget {
    max_tokens: u32,
    used_tokens: Arc<RwLock<u32>>,
    refill_rate_per_minute: u32,
    last_refill: Arc<RwLock<Instant>>,
}

impl TokenBudget {
    pub fn new(max_tokens: u32, refill_rate_per_minute: u32) -> Self {
        Self {
            max_tokens,
            used_tokens: Arc::new(RwLock::new(0)),
            refill_rate_per_minute,
            last_refill: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub fn consume(&self, tokens: u32) -> std::result::Result<(), BudgetExhausted> {
        // Refill if enough time has passed
        self.check_refill();

        let mut used = self.used_tokens.write();
        if *used + tokens > self.max_tokens {
            return Err(BudgetExhausted {
                needed: tokens,
                available: self.max_tokens.saturating_sub(*used),
            });
        }

        *used += tokens;
        Ok(())
    }

    pub fn get_remaining(&self) -> u32 {
        self.max_tokens.saturating_sub(*self.used_tokens.read())
    }

    pub fn get_used(&self) -> u32 {
        *self.used_tokens.read()
    }

    fn check_refill(&self) {
        let mut last = self.last_refill.write();
        let elapsed_minutes = last.elapsed().as_secs() / 60;

        if elapsed_minutes > 0 {
            let refill_amount = (elapsed_minutes as u32).saturating_mul(self.refill_rate_per_minute);
            let mut used = self.used_tokens.write();
            *used = used.saturating_sub(refill_amount);
            *last = Instant::now();
        }
    }
}

impl Clone for TokenBudget {
    fn clone(&self) -> Self {
        Self {
            max_tokens: self.max_tokens,
            used_tokens: Arc::clone(&self.used_tokens),
            refill_rate_per_minute: self.refill_rate_per_minute,
            last_refill: Arc::clone(&self.last_refill),
        }
    }
}

pub struct TokenBudgetManager {
    budgets: Arc<RwLock<HashMap<String, Arc<TokenBudget>>>>,
}

impl TokenBudgetManager {
    pub fn new() -> Self {
        Self {
            budgets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_or_create_budget(
        &self,
        tenant_id: &str,
        max_tokens: u32,
        refill_rate: u32,
    ) -> Arc<TokenBudget> {
        let mut budgets = self.budgets.write();

        budgets
            .entry(tenant_id.to_string())
            .or_insert_with(|| Arc::new(TokenBudget::new(max_tokens, refill_rate)))
            .clone()
    }

    pub fn get_budget(&self, tenant_id: &str) -> Option<Arc<TokenBudget>> {
        self.budgets.read().get(tenant_id).cloned()
    }

    pub fn remove_budget(&self, tenant_id: &str) -> Option<Arc<TokenBudget>> {
        self.budgets.write().remove(tenant_id)
    }

    pub fn list_budgets(&self) -> Vec<(String, u32, u32)> {
        self.budgets
            .read()
            .iter()
            .map(|(id, budget)| (id.clone(), budget.get_used(), budget.get_remaining()))
            .collect()
    }
}

impl Clone for TokenBudgetManager {
    fn clone(&self) -> Self {
        Self {
            budgets: Arc::clone(&self.budgets),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_budget_creation() {
        let budget = TokenBudget::new(1000, 100);
        assert_eq!(budget.get_remaining(), 1000);
        assert_eq!(budget.get_used(), 0);
    }

    #[test]
    fn test_token_consumption() {
        let budget = TokenBudget::new(1000, 100);

        assert!(budget.consume(500).is_ok());
        assert_eq!(budget.get_used(), 500);
        assert_eq!(budget.get_remaining(), 500);
    }

    #[test]
    fn test_budget_exhaustion() {
        let budget = TokenBudget::new(100, 50);

        assert!(budget.consume(50).is_ok());
        assert!(budget.consume(50).is_ok());
        let result = budget.consume(50);
        assert!(result.is_err());

        if let Err(BudgetExhausted { available, .. }) = result {
            assert_eq!(available, 0);
        }
    }

    #[test]
    fn test_budget_manager_creation() {
        let manager = TokenBudgetManager::new();
        let budget = manager.get_or_create_budget("tenant-1", 1000, 100);
        assert_eq!(budget.get_remaining(), 1000);
    }

    #[test]
    fn test_budget_manager_reuse() {
        let manager = TokenBudgetManager::new();

        let budget1 = manager.get_or_create_budget("tenant-1", 1000, 100);
        budget1.consume(100).unwrap();

        let budget2 = manager.get_or_create_budget("tenant-1", 1000, 100);
        assert_eq!(budget2.get_used(), 100);
    }

    #[test]
    fn test_budget_manager_isolation() {
        let manager = TokenBudgetManager::new();

        let budget1 = manager.get_or_create_budget("tenant-1", 1000, 100);
        let budget2 = manager.get_or_create_budget("tenant-2", 500, 50);

        budget1.consume(800).unwrap();

        assert_eq!(budget1.get_used(), 800);
        assert_eq!(budget2.get_used(), 0);
    }

    #[test]
    fn test_budget_get_methods() {
        let manager = TokenBudgetManager::new();
        manager.get_or_create_budget("tenant-1", 1000, 100);

        let budget = manager.get_budget("tenant-1");
        assert!(budget.is_some());

        let missing = manager.get_budget("tenant-missing");
        assert!(missing.is_none());
    }
}
