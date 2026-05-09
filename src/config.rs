use anyhow::Context;
use rig::client::{ModelListingClient, ProviderClient};
use rig::providers::{anthropic, gemini, ollama, openai, openrouter};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, str::FromStr};

use crate::ai::Provider;

#[derive(Serialize, Deserialize, Clone)]
pub struct AiConfig {
    pub provider: Provider,
    pub model: String,
    pub prompt: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: Provider::OpenAI,
            model: "gpt-4o".to_string(),
            prompt: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub ai: AiConfig,
}

impl Config {
    fn merge(self, local: PartialConfig) -> Self {
        Self {
            ai: AiConfig {
                provider: local
                    .ai
                    .as_ref()
                    .and_then(|a| a.provider.clone())
                    .unwrap_or(self.ai.provider),
                model: local
                    .ai
                    .as_ref()
                    .and_then(|a| a.model.clone())
                    .filter(|m| !m.is_empty())
                    .unwrap_or(self.ai.model),
                prompt: local
                    .ai
                    .as_ref()
                    .and_then(|a| a.prompt.clone())
                    .or(self.ai.prompt),
            },
        }
    }
}

#[derive(Deserialize, Default)]
struct PartialConfig {
    ai: Option<PartialAiConfig>,
}

#[derive(Deserialize)]
struct PartialAiConfig {
    provider: Option<Provider>,
    model: Option<String>,
    prompt: Option<String>,
}

pub struct ModelEntry {
    pub display: String,
    pub id: String,
}

pub struct LoadedConfig {
    pub config: Config,
    local_path: PathBuf,
}

impl LoadedConfig {
    pub fn load(repo_root: &PathBuf) -> anyhow::Result<Self> {
        let local_path = local_path(repo_root);
        let global = load_global().unwrap_or_default();
        let local = load_partial(&local_path).unwrap_or_default();
        let merged = global.merge(local);

        let loaded = Self {
            config: merged,
            local_path,
        };
        loaded.save()?;
        Ok(loaded)
    }

    pub fn write_model(&mut self, model: String) -> anyhow::Result<()> {
        self.config.ai.model = model;
        self.save()
    }

    pub fn write_provider(&mut self, provider: String) -> anyhow::Result<()> {
        self.config.ai.provider = Provider::from_str(&provider).context("Invalid provider")?;
        self.save()
    }

    pub async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>> {
        let entries = match self.config.ai.provider {
            Provider::OpenAI => openai::Client::from_env()?
                .list_models()
                .await?
                .into_iter()
                .map(|m| ModelEntry {
                    display: format!("{} ({})", m.display_name(), m.id),
                    id: m.id.to_string(),
                })
                .collect(),
            Provider::Anthropic => anthropic::Client::from_env()?
                .list_models()
                .await?
                .into_iter()
                .map(|m| ModelEntry {
                    display: format!("{} ({})", m.display_name(), m.id),
                    id: m.id.to_string(),
                })
                .collect(),
            Provider::Gemini => gemini::Client::from_env()?
                .list_models()
                .await?
                .into_iter()
                .map(|m| ModelEntry {
                    display: format!("{} ({})", m.display_name(), m.id),
                    id: m.id.to_string(),
                })
                .collect(),
            Provider::Ollama => ollama::Client::from_env()?
                .list_models()
                .await?
                .into_iter()
                .map(|m| ModelEntry {
                    display: format!("{} ({})", m.display_name(), m.id),
                    id: m.id.to_string(),
                })
                .collect(),
            Provider::OpenRouter => openrouter::Client::from_env()?
                .list_models()
                .await?
                .into_iter()
                .map(|m| ModelEntry {
                    display: format!("{} ({})", m.display_name(), m.id),
                    id: m.id.to_string(),
                })
                .collect(),
            _ => {
                return Err(anyhow::anyhow!("Listing is not enabled for this provider"));
            }
        };
        Ok(entries)
    }

    pub fn list_providers() -> Vec<ModelEntry> {
        use strum::IntoEnumIterator;
        Provider::iter()
            .map(|p| ModelEntry {
                display: p.to_string(),
                id: p.to_string(),
            })
            .collect()
    }

    fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.local_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents =
            toml::to_string_pretty(&self.config).context("Could not serialize config")?;
        fs::write(&self.local_path, &contents)
            .with_context(|| format!("Could not write config to {:?}", self.local_path))
    }
}

fn load_global() -> Option<Config> {
    let path = global_path()?;
    let contents = fs::read_to_string(&path).ok()?;
    toml::from_str(&contents).ok()
}

fn load_partial(path: &PathBuf) -> Option<PartialConfig> {
    let contents = fs::read_to_string(path).ok()?;
    toml::from_str(&contents).ok()
}

fn local_path(repo_root: &PathBuf) -> PathBuf {
    repo_root.join(".gmsgconfig.toml")
}

fn global_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "gmsg").map(|dirs| dirs.config_dir().join("config.toml"))
}
