use parking_lot::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,    // Normal operation, requests pass through
    Open,      // Failing, requests rejected (fast-fail)
    HalfOpen,  // Testing with limited traffic
}

pub struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: RwLock<Option<Instant>>,

    failure_threshold: u32,
    success_threshold: u32,
    timeout_duration: Duration,
}

impl CircuitBreaker {
    pub fn new(
        failure_threshold: u32,
        success_threshold: u32,
        timeout_duration: Duration,
    ) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: RwLock::new(None),
            failure_threshold,
            success_threshold,
            timeout_duration,
        }
    }

    pub fn record_success(&self) {
        let mut state = self.state.write();

        match *state {
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::Release);
            }
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::Release) + 1;
                if success_count >= self.success_threshold {
                    *state = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::Release);
                    self.success_count.store(0, Ordering::Release);
                }
            }
            CircuitState::Open => {
                // Ignore successes in Open state until half-open
            }
        }
    }

    pub fn record_failure(&self) {
        let mut state = self.state.write();

        match *state {
            CircuitState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::Release) + 1;
                *self.last_failure_time.write() = Some(Instant::now());

                if failure_count >= self.failure_threshold {
                    *state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open reopens the circuit
                *state = CircuitState::Open;
                *self.last_failure_time.write() = Some(Instant::now());
            }
            CircuitState::Open => {
                *self.last_failure_time.write() = Some(Instant::now());
            }
        }
    }

    pub fn get_state(&self) -> CircuitState {
        let mut state = self.state.write();

        // Transition from Open to HalfOpen if timeout exceeded
        if *state == CircuitState::Open {
            if let Some(last_failure) = *self.last_failure_time.read() {
                if last_failure.elapsed() > self.timeout_duration {
                    *state = CircuitState::HalfOpen;
                    self.success_count.store(0, Ordering::Release);
                }
            }
        }

        *state
    }

    pub fn is_request_allowed(&self) -> bool {
        matches!(
            self.get_state(),
            CircuitState::Closed | CircuitState::HalfOpen
        )
    }

    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Acquire)
    }

    pub fn success_count(&self) -> u32 {
        self.success_count.load(Ordering::Acquire)
    }

    pub fn reset(&self) {
        *self.state.write() = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Release);
        self.success_count.store(0, Ordering::Release);
        *self.last_failure_time.write() = None;
    }
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            state: RwLock::new(*self.state.read()),
            failure_count: AtomicU32::new(self.failure_count.load(Ordering::Acquire)),
            success_count: AtomicU32::new(self.success_count.load(Ordering::Acquire)),
            last_failure_time: RwLock::new(*self.last_failure_time.read()),
            failure_threshold: self.failure_threshold,
            success_threshold: self.success_threshold,
            timeout_duration: self.timeout_duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed_allows_requests() {
        let cb = CircuitBreaker::new(5, 3, Duration::from_secs(60));
        assert!(cb.is_request_allowed());
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, 3, Duration::from_secs(60));

        for _ in 0..3 {
            cb.record_failure();
        }

        assert_eq!(cb.get_state(), CircuitState::Open);
        assert!(!cb.is_request_allowed());
    }

    #[test]
    fn test_circuit_breaker_half_open_after_timeout() {
        let cb = CircuitBreaker::new(1, 3, Duration::from_millis(100));

        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);

        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_closes_after_successes() {
        let cb = CircuitBreaker::new(3, 2, Duration::from_millis(100));

        // Open the circuit
        for _ in 0..3 {
            cb.record_failure();
        }

        assert_eq!(cb.get_state(), CircuitState::Open);

        // Wait for half-open transition
        std::thread::sleep(Duration::from_millis(150));

        // Record successes
        cb.record_success();
        cb.record_success();

        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let cb = CircuitBreaker::new(3, 3, Duration::from_secs(60));

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.failure_count(), 2);

        cb.reset();
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
    }

    #[test]
    fn test_circuit_breaker_clone() {
        let cb1 = CircuitBreaker::new(5, 3, Duration::from_secs(60));
        cb1.record_failure();

        let cb2 = cb1.clone();
        assert_eq!(cb1.failure_count(), cb2.failure_count());
    }
}
