# Changelog
All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.1.2] - 2026-05-15

### Fixed
- Panic when running `gmsg` in a fresh repository with no commits
- Config not being loaded from disk due to incorrect path resolution in `load_local`
- `RateExceeded` error now surfaces the provider's error message

## [0.1.1] - 2026-05-10

### Added
- `gmsg config.provider` — interactively select your AI provider
- `gmsg config.model` — interactively select your model
- `gmsg config.prompt` — set a custom system prompt
- Config file support via `.gmsgconfig.toml` (repo-local overrides global)

### Fixed
- `gmsg --amend` now correctly incorporates currently staged changes

## [0.1.0] - 2026-05-09

### Added
- AI-powered commit message generation
- Interactive TUI editor — review and edit before committing
- `gmsg --amend` to amend the previous commit
- `gmsg --copy` to copy the generated message to clipboard
- `-p / --path` to specify a repo path
- Works from any subdirectory within a repo
- Prints to stdout automatically when piped