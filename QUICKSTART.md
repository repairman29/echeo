# Quick Start - PAYLOAD CLI

## Prerequisites

Install Rust (if not already installed):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## First Run

```bash
cd payload-cli
cargo run
```

This will scan the current directory.

## Scan Your Projects

```bash
# Scan your projects folder
cargo run -- --path ~/projects

# Or use the short flag
cargo run -- -p ~/work
```

## Build for Production

```bash
cargo build --release
```

The binary will be at `target/release/payload` (or `payload.exe` on Windows).

## Install Globally (Optional)

```bash
cargo install --path .
```

Then run `payload` from anywhere.

---

**The Green Waterfall awaits.**

