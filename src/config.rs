use crate::ai::{ModelEntry, Provider, build_model_listing_client};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, path::PathBuf, str::FromStr};

const SYSTEM_PROMPT: &str = r#"
You will be given a git diff. Your task is to generate a commit message that describes ONLY the changes shown in the diff hunks (lines beginning with + or -). 


Be precise. Describe what changed, not what exists around it and infer intent as much as possible.

For small, focused changes keep the body concise. 
Only expand into detail when the change is complex or touches multiple systems and verbosity is deemed neccessary.
You should follow conventional commit specifications 
"#;

trait Merge {
    fn merge(&mut self, other: Self);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiConfig {
    pub provider: Provider,
    pub model: String,
    pub prompt: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: Provider::Gemini,
            model: "gemini-2.0-flash-lite".to_string(),
            prompt: Some(SYSTEM_PROMPT.to_string()),
        }
    }
}

impl Merge for AiConfig {
    fn merge(&mut self, other: Self) {
        self.model = other.model;
        self.provider = other.provider;
        if let Some(prompt) = other.prompt {
            self.prompt = Some(prompt);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Config {
    pub ai: AiConfig,
    #[serde(skip)]
    local: PathBuf,
}

impl Merge for Config {
    fn merge(&mut self, other: Self) {
        self.ai.merge(other.ai);
    }
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let mut config = Self::default();

        if let Some(global) = Self::load_global() {
            config.merge(global);
        }
        if let Some(local) = Self::load_local(path) {
            config.merge(local);
        }
        config.local = path.join(".gmsgconfig.toml");
        config.save()?;
        Ok(config)
    }

    pub fn write_model(&mut self, model: String) -> anyhow::Result<()> {
        self.ai.model = model;
        self.save()
    }

    pub fn write_provider(&mut self, provider: String) -> anyhow::Result<()> {
        self.ai.provider = Provider::from_str(&provider).context("Invalid provider")?;
        self.save()
    }

    pub fn write_prompt(&mut self, prompt: String) -> anyhow::Result<()> {
        self.ai.prompt = Some(prompt);
        self.save()
    }

    pub async fn list_models(provider: Provider) -> anyhow::Result<Vec<ModelEntry>> {
        let client = build_model_listing_client(provider)?;
        client.list_models().await
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
        if let Some(parent) = self.local.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(&self).context("Could not serialize config")?;
        fs::write(&self.local, &contents)
            .with_context(|| format!("Could not write config to {:?}", self.local))
    }

    fn load_global() -> Option<Config> {
        let path = global_path()?;
        let contents = fs::read_to_string(&path).ok()?;
        toml::from_str(&contents).ok()
    }

    fn load_local(path: &Path) -> Option<Config> {
        let local = path.join(".gmsgconfig.toml");
        let contents = fs::read_to_string(&local).ok()?;
        let mut config: Config = toml::from_str(&contents).ok()?;
        config.local = local;
        Some(config)
    }
}

fn global_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "gmsg").map(|dirs| dirs.config_dir().join("config.toml"))
}
