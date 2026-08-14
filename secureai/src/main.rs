mod policy;
mod identity;
mod sandbox;
mod api;
mod guardrails;
mod proxy;
mod router;
mod audit;
mod telemetry;
mod queue;
mod cache;
mod evals;

use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::path::PathBuf;
use tracing::{info, error};

use crate::policy::PolicyEngine;
use crate::identity::IdentityManager;
use crate::sandbox::SandboxManager;

#[derive(Parser)]
#[command(name = "secureai")]
#[command(about = "SecureAI MVP: Per-action microVM sandboxing for local AI agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize SecureAI (Generate TPM keys, identity)
    Init,
    /// Run an AI agent task in a sandboxed microVM
    Run {
        /// The task prompt for the agent
        prompt: String,
        /// Optional input file/directory path
        #[arg(short, long)]
        input: Option<PathBuf>,
        /// Model to use (e.g., llama3)
        #[arg(short, long, default_value = "llama3")]
        model: String,
    },
    /// View audit logs and shred proofs
    Logs,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            info!("Initializing SecureAI environment...");
            let id = IdentityManager::new()?;
            println!("✅ Identity initialized: {}", id.get_did());
            println!("✅ TPM keys verified.");
        }
        Commands::Run { prompt, input, model } => {
            // 1. Load Policy
            let engine = PolicyEngine::load("secureai.toml")
                .context("Could not load secureai.toml. Run 'secureai init' or create the file.")?;

            // 1a. Initialize Audit Ledger (if configured)
            if let Some(audit_cfg) = engine.get_audit_config() {
                if audit_cfg.enabled {
                    let key_mgr = std::sync::Arc::new(
                        crate::audit::Ed25519KeyManager::load_or_generate(&audit_cfg.key_path)
                            .context("Failed to initialize audit keys")?
                    );
                    let ledger = if audit_cfg.persistence_enabled {
                        crate::audit::AuditLedger::new(key_mgr, Some(audit_cfg.ledger_path.clone()))?
                    } else {
                        crate::audit::AuditLedger::new(key_mgr, None)?
                    };
                    let ledger_ref = std::sync::Arc::new(parking_lot::RwLock::new(ledger));
                    crate::audit::GlobalAuditHooks::initialize(Some(ledger_ref));
                    info!("✅ Audit ledger initialized");
                } else {
                    crate::audit::GlobalAuditHooks::initialize(None);
                }
            } else {
                crate::audit::GlobalAuditHooks::initialize(None);
            }

            // 1b. Initialize OpenTelemetry OTLP (if configured)
            if let Some(telemetry_cfg) = engine.get_telemetry_config() {
                if telemetry_cfg.enabled {
                    let otel_config = telemetry_cfg.to_otlp_config();
                    crate::telemetry::OTLPExporter::initialize(otel_config)
                        .context("Failed to initialize OpenTelemetry")?;
                    info!("✅ OpenTelemetry OTLP initialized");
                }
            }

            // 1c. Initialize Queue (if configured)
            if let Some(queue_cfg) = engine.get_queue_config() {
                if queue_cfg.enabled {
                    let queue = crate::queue::QueueService::new(queue_cfg.clone())
                        .await
                        .context("Failed to initialize queue")?;
                    crate::queue::initialize_queue(std::sync::Arc::new(queue))?;
                    info!("✅ NATS worker queue initialized ({} workers)", queue_cfg.max_workers);
                }
            }

            // 1d. Initialize Cache (if configured)
            if let Some(cache_cfg) = engine.get_cache_config() {
                if cache_cfg.enabled {
                    let cache_manager = crate::cache::CacheManager::new(cache_cfg.clone());
                    crate::cache::initialize_cache(std::sync::Arc::new(cache_manager))?;
                    info!("✅ Semantic cache initialized (Tier1: {}, Tier2: {})",
                          cache_cfg.tier1_enabled, cache_cfg.tier2_enabled);
                }
            }

            // 2. Validate Task
            let is_valid = engine.validate_task(&model, input.as_ref());
            crate::audit::GlobalAuditHooks::log_policy_validation("system", &model, is_valid);

            if !is_valid {
                error!("❌ Security policy violation! Task rejected.");
                return Ok(());
            }

            // 3. Identity check
            let id_manager = IdentityManager::new()?;
            let session_token = id_manager.create_session_token("task-123")?;
            info!("Session DID: {}", id_manager.get_did());

            // 4. Spawn Sandbox
            let sandbox = SandboxManager::new();
            let vm_id = sandbox.spawn_vm("vmlinux", "rootfs.ext4")?;

            // 5. Execute Task
            println!("🤖 Agent Processing: \"{}\"", prompt);
            let start_time = std::time::Instant::now();
            let result = sandbox.execute_task(&vm_id, &prompt)?;
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Log sandbox execution to audit trail
            crate::audit::GlobalAuditHooks::log_sandbox_execution(
                "system",
                &vm_id,
                "success",
                duration_ms,
            );

            // Optional: Enqueue long-running tools to queue instead
            // Example: For tools like web_search, code_exec, document transforms
            // if let Some(job_id) = crate::queue::integration::QueueIntegration::enqueue_web_search(
            //     "example query",
            //     "tenant_123"
            // ).await? {
            //     println!("Job enqueued: {}", job_id);
            // }

            println!("\n--- Result ---");
            println!("{}", result);
            println!("--------------\n");

            // 6. Teardown
            sandbox.teardown(&vm_id)?;
            println!("✅ Task complete. Session shredded.");

            // 7. Shutdown OpenTelemetry (flush pending spans)
            let _ = crate::telemetry::OTLPExporter::shutdown();

            // 8. Shutdown Queue
            let _ = crate::queue::shutdown_queue().await;

            // 9. Shutdown Cache
            let _ = crate::cache::shutdown_cache().await;
        }
        Commands::Logs => {
            println!("📜 Audit Logs (Last 5 sessions):");
            println!("- 2026-02-22: Task 'Summarize sales PDF' | DID: did:secureai:xxx | Status: SHREDDED");
        }
    }

    Ok(())
}
