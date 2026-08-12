# SecureAI MVP: Per-Task MicroVM Sandboxing

SecureAI provides a per-action microVM sandboxing environment for local AI agents, mitigating risks like RCE and prompt injection using Firecracker microVMs and TPM-based identity.

## 🚀 Quick Start (WSL2 / Linux)

### 1. Environment Setup
You need a Linux environment with KVM support. If you are on Windows, use WSL2.

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Ollama (for LLM backend)
curl -fsSL https://ollama.com/install.sh | sh

# Install Firecracker binary
# Download from https://github.com/firecracker-microvm/firecracker/releases
# and put it in your PATH.
```

### 2. Prepare Firecracker Assets
Firecracker requires a kernel and a root filesystem.
```bash
# Download a sample kernel and rootfs for testing
wget https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/x86_64/kernels/vmlinux.bin -O vmlinux
wget https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/x86_64/rootfs/bionic.rootfs.ext4 -O rootfs.ext4
```

### 3. Build & Run
```bash
# Clone and build
cd secureai
cargo build --release

# Initialize Identity
./target/release/secureai init

# Run a secure task
./target/release/secureai run "Summarize my docs" --input ./docs --model llama3
```

## 🛡️ Security Architecture
- **Isolation**: Each task runs in a dedicated Firecracker microVM (~125ms boot).
- **Identity**: TPM 2.0 generates DIDs and JWT session tokens for every task.
- **Policy**: TOML-based rules (see `secureai.toml`) define allowed paths and models.
- **Audit**: All actions are logged and attested before the VM is shredded.

## 📂 Project Structure
- `src/main.rs`: CLI and task orchestrator.
- `src/policy.rs`: Policy evaluation engine.
- `src/identity.rs`: TPM/DID identity management.
- `src/sandbox.rs`: Firecracker VM lifecycle manager.
- `secureai.toml`: Security policy configuration.
