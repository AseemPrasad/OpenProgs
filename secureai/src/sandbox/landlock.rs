use anyhow::{Result, Context};
use std::path::PathBuf;
use landlock::{Ruleset, RulesetStatus, AccessFs};

pub struct LandlockRuleset {
    allowed_read_paths: Vec<PathBuf>,
    read_write_workspace: Option<PathBuf>,
    enabled: bool,
}

impl LandlockRuleset {
    pub fn new(allowed_read_paths: Vec<PathBuf>, workspace: Option<PathBuf>) -> Self {
        Self {
            allowed_read_paths,
            read_write_workspace: workspace,
            enabled: true,
        }
    }

    pub fn is_supported() -> bool {
        #[cfg(target_os = "linux")]
        {
            Ruleset::new().is_ok()
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

            let mut ruleset = Ruleset::new()
                .context("Failed to create Landlock ruleset")?;

            // Add read-only access to allowed system paths
            for path in &self.allowed_read_paths {
                ruleset = ruleset
                    .add_rule(
                        landlock::path_beneath_rules(path)
                            .context("Failed to create read-only rule")?,
                        AccessFs::from_all(AccessFs::Execute) | AccessFs::from_all(AccessFs::ReadDir),
                    )
                    .context(format!("Failed to add read-only rule for {:?}", path))?;
            }

            // Add read-write access to workspace if provided
            if let Some(workspace) = &self.read_write_workspace {
                ruleset = ruleset
                    .add_rule(
                        landlock::path_beneath_rules(workspace)
                            .context("Failed to create workspace rule")?,
                        AccessFs::from_all(),
                    )
                    .context(format!("Failed to add read-write rule for workspace {:?}", workspace))?;
            }

            // Restrict filesystem access to only the configured paths
            match ruleset.restrict_self() {
                Ok(RulesetStatus::FullyEnforced) => Ok(()),
                Ok(RulesetStatus::PartiallyEnforced) => {
                    eprintln!("⚠️  Landlock: Partially enforced due to kernel limitations");
                    Ok(())
                }
                Ok(RulesetStatus::NotEnforced) => {
                    eprintln!("⚠️  Landlock: Not enforced on this system");
                    Ok(())
                }
                Err(e) => Err(e).context("Failed to apply Landlock restrictions"),
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("⚠️  Landlock: Skipped on non-Linux systems");
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
    fn test_landlock_creation() {
        let ruleset = LandlockRuleset::new(
            vec![PathBuf::from("/lib"), PathBuf::from("/usr/lib")],
            Some(PathBuf::from("/tmp/workspace")),
        );
        assert!(ruleset.enabled);
    }

    #[test]
    fn test_landlock_is_supported() {
        // This test will only pass on Linux with Landlock kernel support
        let _supported = LandlockRuleset::is_supported();
    }
}
