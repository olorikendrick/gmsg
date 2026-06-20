use crate::ai::{ModelEntry, Provider};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfig {
    pub provider: Provider,
    pub model: ModelEntry,
    pub sys_prompt: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: Provider::Mistral,
            model: ModelEntry {
                id: "mistral-large-latest".to_string(),
                name: Some("Mistral Large".to_string()),
            },
            sys_prompt: "You are an expert git assistant. Write a highly concise, descriptive conventional commit message based on the provided git diff. Do not wrap the output in markdown block text.".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub ai: AiConfig,
}

impl Config {
    fn get_global_path() -> anyhow::Result<PathBuf> {
        let project_dirs = directories::ProjectDirs::from("com", "olorikendrick", "gmsg")
            .ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?;
        Ok(project_dirs.config_dir().join("config.toml"))
    }

    pub fn load(wdir: PathBuf) -> anyhow::Result<Self> {
        let global_config_file = Self::get_global_path()?;
        let default_config = Self::default();

        if !global_config_file.exists() {
            if let Some(parent) = global_config_file.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml_string = toml::to_string_pretty(&default_config)?;
            fs::write(&global_config_file, toml_string)?;
        }

        let local_config_file = wdir.join(".gmsg.toml");

        if !local_config_file.exists() {
            if let Some(parent) = local_config_file.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)?;
                }
            }

            let toml_string = toml::to_string_pretty(&default_config)?;
            fs::write(&local_config_file, toml_string)?;
        }

        let mut builder = config::Config::builder();

        builder = builder.add_source(config::File::from(global_config_file));
        builder = builder.add_source(config::File::from(local_config_file));

        let raw_config = builder.build()?;
        let parsed_config: Self = raw_config.try_deserialize()?;

        Ok(parsed_config)
    }

    pub fn save_to(&self, dir_path: Option<&Path>) -> anyhow::Result<()> {
        let target_path = match dir_path {
            Some(p) => p.join(".gmsg.toml"),
            None => Self::get_global_path()?,
        };

        if let Some(parent) = target_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let toml_string = toml::to_string_pretty(self)?;
        fs::write(target_path, toml_string)?;
        Ok(())
    }
}
