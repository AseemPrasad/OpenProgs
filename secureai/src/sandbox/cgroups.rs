use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use std::os::unix::process::CommandExt;
use nix::unistd::Pid;

pub struct CgroupV2Controller {
    cgroup_path: PathBuf,
    memory_limit_bytes: u64,
    cpu_quota_percent: f64,
    process_limit: u32,
    enabled: bool,
}

impl CgroupV2Controller {
    pub fn new(
        uuid: &str,
        memory_limit_mb: u32,
        cpu_quota: f64,
        process_limit: u32,
    ) -> Result<Self> {
        let cgroup_path = PathBuf::from(format!("/sys/fs/cgroup/secureai-{}", uuid));

        Ok(Self {
            cgroup_path,
            memory_limit_bytes: (memory_limit_mb as u64) * 1024 * 1024,
            cpu_quota_percent: cpu_quota,
            process_limit,
            enabled: true,
        })
    }

    pub fn is_v2_available() -> bool {
        #[cfg(target_os = "linux")]
        {
            Path::new("/sys/fs/cgroup/cgroup.controllers").exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    pub fn create(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            if !self.enabled {
                return Ok(());
            }

            fs::create_dir_all(&self.cgroup_path)
                .context(format!("Failed to create cgroup at {:?}", self.cgroup_path))?;

            // Set memory limit (with swap disabled)
            let memory_max_path = self.cgroup_path.join("memory.max");
            fs::write(&memory_max_path, self.memory_limit_bytes.to_string())
                .context("Failed to set memory.max limit")?;

            // Disable swap
            let memory_swap_path = self.cgroup_path.join("memory.swap.max");
            if memory_swap_path.exists() {
                fs::write(&memory_swap_path, "0")
                    .context("Failed to disable swap")?;
            }

            // Set CPU limits (quota per 100ms period)
            let cpu_quota_path = self.cgroup_path.join("cpu.max");
            let cpu_quota_microseconds = (100_000.0 * self.cpu_quota_percent) as u64;
            let cpu_period = 100_000u64;
            if cpu_quota_path.exists() {
                fs::write(&cpu_quota_path, format!("{} {}", cpu_quota_microseconds, cpu_period))
                    .context("Failed to set cpu.max quota")?;
            }

            // Set process/thread limit
            let procs_max_path = self.cgroup_path.join("pids.max");
            fs::write(&procs_max_path, self.process_limit.to_string())
                .context("Failed to set pids.max limit")?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("⚠️  cgroups v2: Skipped on non-Linux systems");
            Ok(())
        }
    }

    pub fn assign_process(&self, pid: u32) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            if !self.enabled {
                return Ok(());
            }

            let procs_path = self.cgroup_path.join("cgroup.procs");
            fs::write(&procs_path, pid.to_string())
                .context(format!("Failed to assign PID {} to cgroup", pid))?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    pub fn cleanup(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            if !self.enabled || !self.cgroup_path.exists() {
                return Ok(());
            }

            // Try to remove the cgroup directory
            fs::remove_dir(&self.cgroup_path)
                .context(format!("Failed to cleanup cgroup at {:?}", self.cgroup_path))?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn get_path(&self) -> &Path {
        &self.cgroup_path
    }
}

impl Drop for CgroupV2Controller {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgroup_creation() {
        let controller = CgroupV2Controller::new("test-uuid", 512, 1.0, 100)
            .expect("Failed to create controller");
        assert_eq!(controller.memory_limit_bytes, 512 * 1024 * 1024);
        assert_eq!(controller.cpu_quota_percent, 1.0);
    }

    #[test]
    fn test_cgroup_is_available() {
        let _available = CgroupV2Controller::is_v2_available();
    }
}
