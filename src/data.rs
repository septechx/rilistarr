use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum DataError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Guild not configured")]
    GuildNotConfigured,
    #[error("Player already exists")]
    PlayerAlreadyExists,
    #[error("Player not found")]
    PlayerNotFound,
}

pub type Result<T> = std::result::Result<T, DataError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildData {
    #[serde(default)]
    pub leaderboard_channel_id: Option<u64>,
    #[serde(default)]
    pub leaderboard_message_id: Option<u64>,
    #[serde(default = "default_interval")]
    pub update_interval_minutes: u64,
    #[serde(default)]
    pub first_place_role_id: Option<u64>,
    #[serde(default)]
    pub current_first_place_player: Option<String>,
    #[serde(default)]
    pub mod_role_id: Option<u64>,
    #[serde(default)]
    pub players: Vec<String>,
}

fn default_interval() -> u64 {
    30
}

impl Default for GuildData {
    fn default() -> Self {
        Self {
            leaderboard_channel_id: None,
            leaderboard_message_id: None,
            update_interval_minutes: 30,
            first_place_role_id: None,
            current_first_place_player: None,
            mod_role_id: None,
            players: Vec::new(),
        }
    }
}

impl GuildData {
    pub fn data_dir() -> PathBuf {
        PathBuf::from("data")
    }

    pub fn filepath(guild_id: u64) -> PathBuf {
        Self::data_dir().join(format!("{}.json", guild_id))
    }

    pub fn load(guild_id: u64) -> Result<Self> {
        let path = Self::filepath(guild_id);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let data = serde_json::from_str(&content)?;
            Ok(data)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, guild_id: u64) -> Result<()> {
        let dir = Self::data_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let path = Self::filepath(guild_id);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn add_player(&mut self, player_tag: String) -> Result<()> {
        let normalized = normalize_tag(&player_tag);
        if self.players.contains(&normalized) {
            return Err(DataError::PlayerAlreadyExists);
        }
        self.players.push(normalized);
        Ok(())
    }

    pub fn remove_player(&mut self, player_tag: &str) -> Result<()> {
        let normalized = normalize_tag(player_tag);
        let idx = self
            .players
            .iter()
            .position(|p| p == &normalized)
            .ok_or(DataError::PlayerNotFound)?;
        self.players.remove(idx);
        Ok(())
    }

    pub fn set_channel(&mut self, channel_id: u64) {
        self.leaderboard_channel_id = Some(channel_id);
        // Reset message ID when channel changes
        self.leaderboard_message_id = None;
    }

    pub fn set_message_id(&mut self, message_id: u64) {
        self.leaderboard_message_id = Some(message_id);
    }

    pub fn set_interval(&mut self, minutes: u64) {
        self.update_interval_minutes = minutes;
    }

    pub fn set_first_place_role(&mut self, role_id: u64) {
        self.first_place_role_id = Some(role_id);
    }

    pub fn set_mod_role(&mut self, role_id: u64) {
        self.mod_role_id = Some(role_id);
    }

    pub fn set_current_first_place(&mut self, player_tag: Option<String>) {
        self.current_first_place_player = player_tag.map(|t| normalize_tag(&t));
    }

    pub fn is_configured(&self) -> bool {
        self.leaderboard_channel_id.is_some() && !self.players.is_empty()
    }
}

fn normalize_tag(tag: &str) -> String {
    let tag = tag.trim().to_uppercase();
    if tag.starts_with('#') {
        tag
    } else {
        format!("#{}", tag)
    }
}

/// Get all guild IDs that have data files
pub fn get_all_guild_ids() -> Result<Vec<u64>> {
    let dir = GuildData::data_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut guild_ids = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Some(stem) = path.file_stem() {
                if let Ok(id) = stem.to_string_lossy().parse::<u64>() {
                    guild_ids.push(id);
                }
            }
        }
    }
    Ok(guild_ids)
}
