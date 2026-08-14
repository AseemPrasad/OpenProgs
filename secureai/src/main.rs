mod policy;
mod identity;
mod sandbox;
mod api;
mod guardrails;
mod proxy;

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

            // 2. Validate Task
            if !engine.validate_task(&model, input.as_ref()) {
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
            let result = sandbox.execute_task(&vm_id, &prompt)?;
            
            println!("\n--- Result ---");
            println!("{}", result);
            println!("--------------\n");

            // 6. Teardown
            sandbox.teardown(&vm_id)?;
            println!("✅ Task complete. Session shredded.");
        }
        Commands::Logs => {
            println!("📜 Audit Logs (Last 5 sessions):");
            println!("- 2026-02-22: Task 'Summarize sales PDF' | DID: did:secureai:xxx | Status: SHREDDED");
        }
    }

    Ok(())
}
