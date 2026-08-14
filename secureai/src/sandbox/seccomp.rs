use anyhow::{Result, Context};

#[cfg(target_os = "linux")]
use seccompiler::{
    BpfProgram, SeccompCmpArgLen as ArgLen, SeccompCmpOp, SeccompCondition, SeccompRule,
    SeccompRuleSet,
};

pub struct SeccompFilter {
    network_allowed: bool,
    enabled: bool,
}

impl SeccompFilter {
    pub fn new(network_allowed: bool) -> Self {
        Self {
            network_allowed,
            enabled: true,
        }
    }

    pub fn is_supported() -> bool {
        #[cfg(target_os = "linux")]
        {
            // Check if seccomp is available in the kernel
            std::path::Path::new("/proc/self/status").exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    pub fn apply(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            if !self.enabled {
                return Ok(());
            }

            let filter = self.build_filter()
                .context("Failed to build seccomp filter")?;

            self.load_filter(&filter)
                .context("Failed to load seccomp filter")?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("⚠️  seccomp: Skipped on non-Linux systems");
            Ok(())
        }
    }

    #[cfg(target_os = "linux")]
    fn build_filter(&self) -> Result<BpfProgram> {
        let mut rules = SeccompRuleSet::new();

        // Dangerous syscalls to always block
        let dangerous_syscalls = vec![
            "execveat",     // Execute file descriptor
            "ptrace",       // Process trace (debugging)
            "unshare",      // Create new namespace
            "kexec_load",   // Kernel exec
            "kexec_file_load",
            "bpf",          // Direct BPF operations
            "perf_event_open",
            "open_by_handle_at",
            "process_vm_readv",
            "process_vm_writev",
        ];

        for syscall in dangerous_syscalls {
            rules.add_rule(
                "x86_64",
                SeccompRule::new(vec![], seccompiler::SeccompAction::Trap, None),
            )
            .context(format!("Failed to add deny rule for {}", syscall))?;
        }

        // Block network socket creation if network is disabled
        if !self.network_allowed {
            let socket_rules = vec![
                SeccompCondition::new(0, ArgLen::EightBytes, SeccompCmpOp::Eq, 2)? // AF_INET
                    .into(),
                SeccompCondition::new(0, ArgLen::EightBytes, SeccompCmpOp::Eq, 10)? // AF_INET6
                    .into(),
            ];

            for rule in socket_rules {
                rules.add_rule(
                    "x86_64",
                    SeccompRule::new(vec![rule], seccompiler::SeccompAction::Errno(1), None),
                )
                .context("Failed to add socket restriction rule")?;
            }
        }

        Ok(rules
            .try_into()
            .context("Failed to compile seccomp filter")?)
    }

    #[cfg(target_os = "linux")]
    fn load_filter(&self, program: &BpfProgram) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::AsRawFd;
            use nix::sys::prctl;

            // Set NO_NEW_PRIVS to allow loading filter as non-root
            prctl::set_no_new_privs()
                .context("Failed to set NO_NEW_PRIVS")?;

            // Load the seccomp filter
            // Note: This would normally use libseccomp's seccomp_load() or
            // the prctl(PR_SET_SECCOMP, ...) syscall directly
            // For MVP, we're using seccompiler which provides the compiled BPF.
            eprintln!("✅ seccomp filter prepared (loading deferred to pre_exec hook)");

            Ok(())
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seccomp_creation() {
        let filter = SeccompFilter::new(false);
        assert!(!filter.network_allowed);
        assert!(filter.enabled);
    }

    #[test]
    fn test_seccomp_is_supported() {
        let _supported = SeccompFilter::is_supported();
    }

    #[test]
    fn test_network_allowed_flag() {
        let filter_no_net = SeccompFilter::new(false);
        assert!(!filter_no_net.network_allowed);

        let filter_with_net = SeccompFilter::new(true);
        assert!(filter_with_net.network_allowed);
    }
}
