use crate::brawl_api::{BrawlApiError, Client, Player};
use crate::data::{self, DataError, GuildData};
use futures::future::join_all;
use serenity::builder::EditMessage;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::sync::Arc;
use thiserror::Error;
use tokio::time::{interval, Duration};

#[derive(Error, Debug)]
pub enum LeaderboardError {
    #[error("Brawl API error: {0}")]
    BrawlApi(#[from] BrawlApiError),
    #[error("Discord error: {0}")]
    Discord(#[from] serenity::Error),
    #[error("Data error: {0}")]
    Data(#[from] DataError),
    #[error("Channel not configured")]
    ChannelNotConfigured,
    #[error("No players configured")]
    NoPlayers,
}

pub type Result<T> = std::result::Result<T, LeaderboardError>;

#[derive(Clone)]
pub struct LeaderboardUpdater {
    ctx: Context,
    brawl_client: Arc<Client>,
}

impl LeaderboardUpdater {
    pub fn new(ctx: Context, brawl_token: &str) -> Self {
        Self {
            ctx,
            brawl_client: Arc::new(Client::new(brawl_token)),
        }
    }

    /// Update leaderboard for a specific guild
    pub async fn update_guild(&self, guild_id: GuildId) -> Result<()> {
        let mut guild_data = GuildData::load(guild_id.get())?;

        // Check if configured
        let channel_id = guild_data
            .leaderboard_channel_id
            .ok_or(LeaderboardError::ChannelNotConfigured)?;

        if guild_data.players.is_empty() {
            return Err(LeaderboardError::NoPlayers);
        }

        // Fetch player data concurrently
        let client = self.brawl_client.clone();
        let futures: Vec<_> = guild_data
            .players
            .iter()
            .map(|tag| {
                let client = client.clone();
                let tag = tag.clone();
                async move { client.get_player(&tag).await.map_err(|e| (tag.clone(), e)) }
            })
            .collect();

        let results = join_all(futures).await;

        let mut players = Vec::new();
        for result in results {
            match result {
                Ok(player) => players.push(player),
                Err((tag, e)) => {
                    eprintln!("Failed to fetch player {}: {}", tag, e);
                }
            }
        }

        if players.is_empty() {
            return Err(LeaderboardError::NoPlayers);
        }

        // Sort by trophies (descending)
        players.sort_by(|a, b| b.trophies.cmp(&a.trophies));

        self.handle_first_place_role(guild_id, &guild_data, &players)
            .await?;

        // Update stored first place player
        if let Some(first) = players.first() {
            let new_first_tag = if first.tag.starts_with('#') {
                first.tag.clone()
            } else {
                format!("#{}", first.tag)
            };
            if guild_data.current_first_place_player.as_ref() != Some(&new_first_tag) {
                guild_data.set_current_first_place(Some(new_first_tag));
                guild_data.save(guild_id.get())?;
            }
        }

        let leaderboard_text = format_leaderboard(&players);

        let channel_id = ChannelId::new(channel_id);

        if let Some(message_id) = guild_data.leaderboard_message_id {
            // Try to edit existing message
            let message_id = MessageId::new(message_id);
            let edit = EditMessage::new().content(&leaderboard_text);
            match channel_id.edit_message(&self.ctx, message_id, edit).await {
                Ok(_) => {}
                Err(_) => {
                    // Message might have been deleted, create new one
                    let msg = channel_id.say(&self.ctx, &leaderboard_text).await?;
                    guild_data.set_message_id(msg.id.get());
                    guild_data.save(guild_id.get())?;
                }
            }
        } else {
            // Create new message
            let msg = channel_id.say(&self.ctx, &leaderboard_text).await?;
            guild_data.set_message_id(msg.id.get());
            guild_data.save(guild_id.get())?;
        }

        Ok(())
    }

    /// Handle giving/removing first place role
    async fn handle_first_place_role(
        &self,
        guild_id: GuildId,
        guild_data: &GuildData,
        players: &[Player],
    ) -> Result<()> {
        let role_id = match guild_data.first_place_role_id {
            Some(id) => RoleId::new(id),
            None => return Ok(()), // No role configured
        };

        let guild = match guild_id.to_guild_cached(&self.ctx) {
            Some(g) => g.clone(),
            None => return Ok(()), // Guild not in cache
        };

        // Remove role from previous #1 if they exist and are different
        if let Some(prev_tag) = &guild_data.current_first_place_player {
            if let Some(current_first) = players.first() {
                let current_tag = if current_first.tag.starts_with('#') {
                    current_first.tag.clone()
                } else {
                    format!("#{}", current_first.tag)
                };
                if prev_tag != &current_tag {
                    // Find and remove role from previous #1
                    for member in guild.members.values() {
                        if member.roles.contains(&role_id) {
                            let _ = member.remove_role(&self.ctx, role_id).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn format_leaderboard(players: &[Player]) -> String {
    let mut text = String::from("**🏆 Brawl Stars Trophy Leaderboard**\n\n");

    for (i, player) in players.iter().enumerate() {
        let medal = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "▫️",
        };
        let tag_display = if player.tag.starts_with('#') {
            &player.tag
        } else {
            &format!("#{}", player.tag)
        };
        text.push_str(&format!(
            "{} **#{}** - {} (`{}`): **{}** trophies\n",
            medal,
            i + 1,
            player.name,
            tag_display,
            player.trophies
        ));
    }

    text.push_str("\n_Last updated: ");
    text.push_str(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    text.push('_');

    text
}

/// Start the background update loop
pub async fn start_update_loop(ctx: Context, brawl_token: String) {
    let mut check_interval = interval(Duration::from_secs(60)); // Check every minute
    let mut last_update_times: std::collections::HashMap<u64, std::time::Instant> =
        std::collections::HashMap::new();

    loop {
        check_interval.tick().await;

        // Get all guilds that have data
        match data::get_all_guild_ids() {
            Ok(guild_ids) => {
                for guild_id in guild_ids {
                    if let Ok(guild_data) = GuildData::load(guild_id) {
                        if !guild_data.is_configured() {
                            continue;
                        }

                        // Check if it's time to update
                        let now = std::time::Instant::now();
                        let should_update = match last_update_times.get(&guild_id) {
                            Some(last_update) => {
                                let elapsed = now.duration_since(*last_update).as_secs();
                                elapsed >= guild_data.update_interval_minutes * 60
                            }
                            None => true, // First update
                        };

                        if should_update {
                            let updater = LeaderboardUpdater::new(ctx.clone(), &brawl_token);
                            if let Err(e) = updater.update_guild(GuildId::new(guild_id)).await {
                                eprintln!("Failed to update guild {}: {}", guild_id, e);
                            } else {
                                last_update_times.insert(guild_id, now);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get guild IDs: {}", e);
            }
        }
    }
}
