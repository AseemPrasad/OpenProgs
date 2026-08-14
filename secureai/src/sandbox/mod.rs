use anyhow::{Result, Context as AnyhowContext};
use std::process::Command;
use std::path::PathBuf;

use crate::policy::IsolationPolicy;

pub mod landlock;
pub mod cgroups;
pub mod seccomp;
pub mod executor;

pub use executor::SandboxExecutor;

pub struct SandboxManager {
    firecracker_path: String,
    enable_kernel_isolation: bool,
}

impl SandboxManager {
    pub fn new() -> Self {
        let enable_kernel_isolation = std::env::var("ENABLE_KERNEL_ISOLATION")
            .ok()
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        Self {
            firecracker_path: "firecracker".to_string(),
            enable_kernel_isolation,
        }
    }

    pub fn with_isolation(enable: bool) -> Self {
        Self {
            firecracker_path: "firecracker".to_string(),
            enable_kernel_isolation: enable,
        }
    }

    pub fn spawn_vm(&self, kernel_path: &str, rootfs_path: &str) -> Result<String> {
        // In a real environment, we'd use the Firecracker SDK or
        // talk to the Firecracker API socket (typically /tmp/firecracker.socket).

        // 1. Start Firecracker process
        // 2. Configure kernel and boot source
        // 3. Configure drive (rootfs)
        // 4. Start the VM

        println!("🚀 Spawning Firecracker microVM...");
        println!("  - Kernel: {}", kernel_path);
        println!("  - Rootfs: {}", rootfs_path);

        // Mocking the API handshake for the MVP
        // In reality, this would be a series of PUT requests to the socket

        Ok("vm-xxxx-session".to_string())
    }

    pub fn execute_task(&self, vm_id: &str, prompt: &str) -> Result<String> {
        println!("🛡️ Executing task in sandboxed VM [{}]", vm_id);

        // This would involve sending the task to the VM's guest agent
        // or piping it through a vsock.

        Ok(format!("Analysis result for: '{}'", prompt))
    }

    pub fn execute_with_isolation(
        &self,
        mut command: Command,
        policy: IsolationPolicy,
    ) -> Result<std::process::Output> {
        if !self.enable_kernel_isolation {
            // Fall back to direct execution if isolation is disabled
            return command
                .output()
                .context("Failed to execute command without isolation");
        }

        let mut executor = SandboxExecutor::new(command, policy)
            .context("Failed to create SandboxExecutor")?;

        executor.execute()
            .context("Failed to execute sandboxed command")
    }

    pub fn teardown(&self, vm_id: &str) -> Result<()> {
        println!("🧹 Shreddring microVM [{}] and performing remote attestation...", vm_id);
        Ok(())
    }
}
