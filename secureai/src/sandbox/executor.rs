use anyhow::{Result, Context};
use std::process::{Command, Stdio, Output};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::policy::IsolationPolicy;
use super::landlock::LandlockRuleset;
use super::cgroups::CgroupV2Controller;
use super::seccomp::SeccompFilter;

pub struct SandboxExecutor {
    command: Command,
    isolation_policy: IsolationPolicy,
    workspace_path: PathBuf,
    uuid: String,
}

impl SandboxExecutor {
    pub fn new(
        mut command: Command,
        isolation_policy: IsolationPolicy,
    ) -> Result<Self> {
        let uuid = Uuid::new_v4().to_string();

        let workspace_path = if let Some(workspace_template) = &isolation_policy.workspace_path {
            PathBuf::from(workspace_template.replace("{uuid}", &uuid))
        } else {
            PathBuf::from(format!("/tmp/secureai-{}", uuid))
        };

        // Create workspace directory
        std::fs::create_dir_all(&workspace_path)
            .context(format!("Failed to create workspace at {:?}", workspace_path))?;

        Ok(Self {
            command,
            isolation_policy,
            workspace_path,
            uuid,
        })
    }

    pub fn execute(&mut self) -> Result<Output> {
        let isolation_policy = &self.isolation_policy;

        #[cfg(target_os = "linux")]
        {
            // Prepare cgroups
            if isolation_policy.enable_cgroups {
                let mut cgroup = CgroupV2Controller::new(
                    &self.uuid,
                    isolation_policy.memory_limit_mb,
                    isolation_policy.cpu_quota,
                    isolation_policy.max_processes,
                )?;

                cgroup.create()
                    .context("Failed to create cgroup")?;

                // We'll use pre_exec to assign process to cgroup
                let cgroup_path = cgroup.get_path().to_path_buf();
                unsafe {
                    self.command.pre_exec(move || {
                        // This runs in the child process before exec()
                        if let Ok(pid) = nix::unistd::getpid().as_raw() {
                            let _ = std::fs::write(
                                cgroup_path.join("cgroup.procs"),
                                pid.to_string(),
                            );
                        }
                        Ok(())
                    });
                }
            }

            // Prepare Landlock
            if isolation_policy.enable_landlock {
                let mut landlock_paths = isolation_policy.landlock_paths.clone();
                // Always include the workspace
                landlock_paths.push(self.workspace_path.clone());

                let landlock_ruleset = LandlockRuleset::new(
                    landlock_paths,
                    Some(self.workspace_path.clone()),
                );

                unsafe {
                    self.command.pre_exec(move || {
                        landlock_ruleset.apply()
                            .context("Failed to apply Landlock restrictions")?;
                        Ok(())
                    });
                }
            }

            // Prepare seccomp
            if isolation_policy.enable_seccomp {
                let seccomp = SeccompFilter::new(isolation_policy.cpu_quota > 0.0);

                unsafe {
                    self.command.pre_exec(move || {
                        seccomp.apply()
                            .context("Failed to apply seccomp filter")?;
                        Ok(())
                    });
                }
            }
        }

        // Execute command with all isolation mechanisms applied
        let output = self.command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute sandboxed command")?;

        Ok(output)
    }

    pub fn get_workspace(&self) -> &Path {
        &self.workspace_path
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn cleanup(&self) -> Result<()> {
        if self.workspace_path.exists() {
            std::fs::remove_dir_all(&self.workspace_path)
                .context(format!("Failed to cleanup workspace at {:?}", self.workspace_path))?;
        }
        Ok(())
    }
}

impl Drop for SandboxExecutor {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command as StdCommand;

    #[test]
    fn test_executor_creation() {
        let command = StdCommand::new("echo");
        let policy = IsolationPolicy {
            enable_landlock: false,
            enable_seccomp: false,
            enable_cgroups: false,
            landlock_paths: vec![],
            workspace_path: None,
            memory_limit_mb: 512,
            cpu_quota: 1.0,
            max_processes: 100,
        };

        let executor = SandboxExecutor::new(command, policy)
            .expect("Failed to create executor");

        assert!(!executor.uuid.is_empty());
        assert!(executor.workspace_path.exists());
    }

    #[test]
    fn test_executor_workspace_exists() {
        let command = StdCommand::new("true");
        let policy = IsolationPolicy {
            enable_landlock: false,
            enable_seccomp: false,
            enable_cgroups: false,
            landlock_paths: vec![],
            workspace_path: None,
            memory_limit_mb: 512,
            cpu_quota: 1.0,
            max_processes: 100,
        };

        let executor = SandboxExecutor::new(command, policy)
            .expect("Failed to create executor");

        let workspace = executor.get_workspace();
        assert!(workspace.exists(), "Workspace should be created");
    }
}
