# Changelog
## [0.1.3] - 2026-05-31
### Added
- `MockAi` provider for deterministic, offline testing without real API calls

### Changed
- Model listing abstracted behind a `ListModels` trait for improved testability
- Output handling centralized via `OutputAction`, with message attached directly to each variant
- `build_commit_agent` now uses a borrowed `&dyn GenerateCommitMsg` instead of `Box<dyn>`, reducing heap allocation
- `Config::load` and `wdir` initialized once in `run()` and passed down, removing redundant calls

### Fixed
- Typo in `SYSTEM_PROMPT` constant (`necesary` → `necessary`)
- Clipboard feature gated behind a compile-time `cfg` flag, fixing a crash on platforms
  without a system clipboard (e.g. Termux)

### Tests
- Unit tests for provider parsing across all `Provider` variants
- Client construction test (`build_model_listing_client_works`) without real API keys
- Coverage for `get_staged_files` — staged files and empty index cases
- Config loading and merging tests (`AiConfig::merge`, `Config::load`, `Config::write_provider`, `Config::list_providers`)
- Local config override verification (`test_local_config_is_loaded`)
- Clipboard output test via `test_c_flag_works`
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
