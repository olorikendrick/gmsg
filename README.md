[![Crates.io](https://img.shields.io/crates/v/gmsg.svg?color=orange)](https://crates.io/crates/gmsg)
[![Downloads](https://img.shields.io/crates/d/gmsg.svg)](https://crates.io/crates/gmsg)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Built with Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
# GMSG
**AI-powered utility for generating conventional Git commit messages.**

`gmsg` is a high-performance CLI tool built in Rust for generating commit messages. It uses AI to analyze your staged diffs and generate messages based on the Conventional Commits specification.

---

## 🚀 Features

- **Spec-Grounded:** Uses the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0) specification as a system prompt to ensure total compliance. You can also modify the system prompt to align with your preferred specifications.
- **Interactive TUI:** Review and edit generated messages in a [Ratatui](https://ratatui.rs/)-powered editor before finalizing.
- **UNIX Compliant:** Automatically discovers the closest git repository in your current folder with TTY/pipe-aware behavior.
- **Clipboard & Amend Support:** Easily copy messages to your clipboard or amend the most recent commit.
- **Multi-Provider and Model Support:** Built with [Rig](https://rig.rs/), providing excellent support for a wide range of LLM providers and models of your choice.

---

## 🛠 Installation

*Ensure you have your appropriate API key set in your environment variables.*

```bash
export GEMINI_API_KEY="your_api_key_here"
cargo install gmsg
```

Or download a [prebuilt binary](https://github.com/olorikendrick/gmsg/releases/latest) for your platform.
---

## 📖 Usage

### Standard Workflow

Stage your changes and let `gmsg` handle the whole commit process.

```bash
git add .
gmsg
```

### Interactive Review

Review and modify the generated message before committing:

```bash
gmsg -i
```

- **Ctrl+S:** Save and continue.
- **Ctrl+Q:** Discard and exit.

### Helper Mode

If you just want to generate the message without committing:

```bash
# Copy to clipboard and exit
gmsg -c

# Output to a file
gmsg > message.txt

# Pipe to another utility
gmsg | grep
```

### Amending

Amend the message of your last commit. If you have staged changes, the diff is sent alongside the previous message to the AI. Otherwise, it opens an editor.

```bash
gmsg -a
```

---

## ⚙️ Configuration

`gmsg` is zero-config by default.
But you can configure it.

```bash
gmsg config.provider   # set your LLM provider
gmsg config.model      # set your model
gmsg config.prompt <Prompt>    # customize the system prompt
```
<img width="400" height="225" alt="2026-05-10 17-48-47" src="https://github.com/user-attachments/assets/fe77704f-4b55-4e45-bd03-a894a4706824" />

| Flag | Long | Description |
| :--- | :--- | :--- |
| `-p` | `--path` | Path to the repository (defaults to current dir). |
| `-i` | `--interactive` | Opens the TUI editor before committing. |
| `-c` | `--copy` | Copies the message to clipboard and exits. |
| `-a` | `--amend` | Amends the HEAD commit with the new message. |

Configuration can be set in your project's `.gmsgconfig.toml` or your global config directory. Project-level config takes precedence.

---

## 🏗 Architecture

- **Agent Logic:** Powered by the [`rig`](https://rig.rs/) crate for LLM orchestration.
- **Git Operations:** Uses [`git2-rs`](https://github.com/rust-lang/git2-rs) for robust interaction with Git.
- **Terminal UI:** Built with [`ratatui`](https://ratatui.rs/) and `ratatui-textarea` for a smooth editing experience.
- **Async Runtime:** Driven by [`tokio`](https://tokio.rs/) for non-blocking AI generation.

---

[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

## 🛡 License

MIT – Build something great.
