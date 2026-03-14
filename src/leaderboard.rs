use crate::brawl_api::{BrawlApiError, Clan, Client, Player};
use crate::data::{self, DataError, GuildData};
use crate::leaderboard_image;
use crate::serenity::CreateAttachment;
use futures::future::join_all;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::fmt::Debug;
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
    #[error("Image error: {0}")]
    Image(#[from] Box<dyn std::error::Error>),
    #[error("Channel not configured")]
    ChannelNotConfigured,
    #[error("No players configured")]
    NoPlayers,
    #[error("No clans configured")]
    NoClans,
}

pub type Result<T> = std::result::Result<T, LeaderboardError>;

pub trait Leaderboard: Clone + Send + Sync + 'static {
    type Entity: Clone + Debug + Send + Sync;

    fn tags<'a>(&self, guild_data: &'a GuildData) -> &'a [String];
    async fn fetch_all(&self, client: &Client, tags: &[String]) -> Vec<Self::Entity>;
    fn sort(entities: &mut [Self::Entity]);
    fn get_first_tag(entity: &Self::Entity) -> String;
    fn set_first_place(&self, guild_data: &mut GuildData, tag: String);
    fn no_data_error(&self) -> LeaderboardError;
    fn get_message_id(&self, guild_data: &GuildData) -> Option<u64>;
    fn set_message_id(&self, guild_data: &mut GuildData, id: u64);
    fn get_first_place<'a>(&self, guild_data: &'a GuildData) -> Option<&'a String>;
    fn title(&self) -> &str;
    fn to_entries(entities: &[Self::Entity]) -> Vec<LeaderboardEntry>;
}

#[derive(Clone, Debug)]
pub struct LeaderboardEntry {
    pub name: String,
    pub trophies: i32,
    pub member_count: Option<u32>,
}

impl LeaderboardEntry {
    pub fn from_player(player: &Player) -> Self {
        Self {
            name: player.name.clone(),
            trophies: player.trophies,
            member_count: None,
        }
    }

    pub fn from_clan(clan: &Clan) -> Self {
        Self {
            name: clan.name.clone(),
            trophies: clan.trophies,
            member_count: Some(clan.members.len() as u32),
        }
    }
}

#[derive(Clone)]
pub struct PlayerLeaderboard;

impl PlayerLeaderboard {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle_first_place_role(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        guild_data: &GuildData,
        players: &[Player],
    ) -> Result<()> {
        let role_id = match guild_data.first_place_role_id {
            Some(id) => RoleId::new(id),
            None => return Ok(()),
        };

        let guild = match guild_id.to_guild_cached(ctx) {
            Some(g) => g.clone(),
            None => return Ok(()),
        };

        if let Some(prev_tag) = &guild_data.current_first_place_player {
            if let Some(current_first) = players.first() {
                let current_tag = if current_first.tag.starts_with('#') {
                    current_first.tag.clone()
                } else {
                    format!("#{}", current_first.tag)
                };
                if prev_tag != &current_tag {
                    for member in guild.members.values() {
                        if member.roles.contains(&role_id) {
                            let _ = member.remove_role(ctx, role_id).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Leaderboard for PlayerLeaderboard {
    type Entity = Player;

    fn tags<'a>(&self, guild_data: &'a GuildData) -> &'a [String] {
        &guild_data.players
    }

    async fn fetch_all(&self, client: &Client, tags: &[String]) -> Vec<Player> {
        let futures: Vec<_> = tags
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
        players
    }

    fn sort(entities: &mut [Player]) {
        entities.sort_by(|a, b| b.trophies.cmp(&a.trophies));
    }

    fn get_first_tag(entity: &Player) -> String {
        if entity.tag.starts_with('#') {
            entity.tag.clone()
        } else {
            format!("#{}", entity.tag)
        }
    }

    fn set_first_place(&self, guild_data: &mut GuildData, tag: String) {
        guild_data.set_current_first_place(Some(tag));
    }

    fn no_data_error(&self) -> LeaderboardError {
        LeaderboardError::NoPlayers
    }

    fn get_message_id(&self, guild_data: &GuildData) -> Option<u64> {
        guild_data.leaderboard_message_id
    }

    fn set_message_id(&self, guild_data: &mut GuildData, id: u64) {
        guild_data.set_message_id(id);
    }

    fn get_first_place<'a>(&self, guild_data: &'a GuildData) -> Option<&'a String> {
        guild_data.current_first_place_player.as_ref()
    }

    fn title(&self) -> &str {
        "Brawl Stars Trophy Leaderboard"
    }

    fn to_entries(entities: &[Player]) -> Vec<LeaderboardEntry> {
        entities.iter().map(LeaderboardEntry::from_player).collect()
    }
}

#[derive(Clone)]
pub struct ClanLeaderboard;

impl ClanLeaderboard {
    pub fn new() -> Self {
        Self
    }
}

impl Leaderboard for ClanLeaderboard {
    type Entity = Clan;

    fn tags<'a>(&self, guild_data: &'a GuildData) -> &'a [String] {
        &guild_data.clans
    }

    async fn fetch_all(&self, client: &Client, tags: &[String]) -> Vec<Clan> {
        let futures: Vec<_> = tags
            .iter()
            .map(|tag| {
                let client = client.clone();
                let tag = tag.clone();
                async move { client.get_clan(&tag).await.map_err(|e| (tag.clone(), e)) }
            })
            .collect();

        let results = join_all(futures).await;

        let mut clans = Vec::new();
        for result in results {
            match result {
                Ok(clan) => clans.push(clan),
                Err((tag, e)) => {
                    eprintln!("Failed to fetch clan {}: {}", tag, e);
                }
            }
        }
        clans
    }

    fn sort(entities: &mut [Clan]) {
        entities.sort_by(|a, b| b.trophies.cmp(&a.trophies));
    }

    fn get_first_tag(entity: &Clan) -> String {
        if entity.tag.starts_with('#') {
            entity.tag.clone()
        } else {
            format!("#{}", entity.tag)
        }
    }

    fn set_first_place(&self, guild_data: &mut GuildData, tag: String) {
        guild_data.set_current_first_place_clan(Some(tag));
    }

    fn no_data_error(&self) -> LeaderboardError {
        LeaderboardError::NoClans
    }

    fn get_message_id(&self, guild_data: &GuildData) -> Option<u64> {
        guild_data.clan_leaderboard_message_id
    }

    fn set_message_id(&self, guild_data: &mut GuildData, id: u64) {
        guild_data.set_clan_message_id(id);
    }

    fn get_first_place<'a>(&self, guild_data: &'a GuildData) -> Option<&'a String> {
        guild_data.current_first_place_clan.as_ref()
    }

    fn title(&self) -> &str {
        "Brawl Stars Clan Leaderboard"
    }

    fn to_entries(entities: &[Clan]) -> Vec<LeaderboardEntry> {
        entities.iter().map(LeaderboardEntry::from_clan).collect()
    }
}

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

    pub async fn update_player_leaderboard(&self, guild_id: GuildId) -> Result<()> {
        let leaderboard = PlayerLeaderboard::new();
        self.update_leaderboard(&leaderboard, guild_id).await?;

        let guild_data = GuildData::load(guild_id.get())?;
        let tags = leaderboard.tags(&guild_data);
        let players = leaderboard.fetch_all(&self.brawl_client, tags).await;
        leaderboard
            .handle_first_place_role(&self.ctx, guild_id, &guild_data, &players)
            .await?;

        Ok(())
    }

    pub async fn update_clan_leaderboard(&self, guild_id: GuildId) -> Result<()> {
        let leaderboard = ClanLeaderboard::new();
        self.update_leaderboard(&leaderboard, guild_id).await?;

        Ok(())
    }

    async fn update_leaderboard<L: Leaderboard>(
        &self,
        leaderboard: &L,
        guild_id: GuildId,
    ) -> Result<()> {
        let mut guild_data = GuildData::load(guild_id.get())?;

        let channel_id = guild_data
            .leaderboard_channel_id
            .ok_or(LeaderboardError::ChannelNotConfigured)?;

        let tags = leaderboard.tags(&guild_data);
        if tags.is_empty() {
            return Err(leaderboard.no_data_error());
        }

        let entities = leaderboard.fetch_all(&self.brawl_client, tags).await;

        if entities.is_empty() {
            return Err(leaderboard.no_data_error());
        }

        let mut entities = entities;
        L::sort(&mut entities);

        if let Some(first) = entities.first() {
            let new_first_tag = L::get_first_tag(first);
            if leaderboard.get_first_place(&guild_data).map(|s| s.as_str())
                != Some(new_first_tag.as_str())
            {
                leaderboard.set_first_place(&mut guild_data, new_first_tag);
                guild_data.save(guild_id.get())?;
            }
        }

        let entries = L::to_entries(&entities);
        let title = leaderboard.title();

        let image_bytes = leaderboard_image::render_leaderboard_image(&entries, title)?;

        let channel_id = ChannelId::new(channel_id);

        if let Some(message_id) = leaderboard.get_message_id(&guild_data) {
            let message_id = MessageId::new(message_id);
            let attachment = CreateAttachment::bytes(image_bytes.clone(), "leaderboard.png");
            let edit = serenity::builder::EditMessage::new()
                .new_attachment(attachment);
            match channel_id.edit_message(&self.ctx, message_id, edit).await {
                Ok(_) => {}
                Err(_) => {
                    let msg = channel_id
                        .send_message(
                            &self.ctx,
                            serenity::builder::CreateMessage::new()
                                .add_file(CreateAttachment::bytes(
                                    image_bytes.clone(),
                                    "leaderboard.png",
                                )),
                        )
                        .await?;
                    leaderboard.set_message_id(&mut guild_data, msg.id.get());
                    guild_data.save(guild_id.get())?;
                }
            }
        } else {
            let msg = channel_id
                .send_message(
                    &self.ctx,
                    serenity::builder::CreateMessage::new()
                        .add_file(CreateAttachment::bytes(
                            image_bytes,
                            "leaderboard.png",
                        )),
                )
                .await?;
            leaderboard.set_message_id(&mut guild_data, msg.id.get());
            guild_data.save(guild_id.get())?;
        }

        Ok(())
    }
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

                            // Update player leaderboard if configured
                            if guild_data.is_player_configured() {
                                if let Err(e) = updater
                                    .update_player_leaderboard(GuildId::new(guild_id))
                                    .await
                                {
                                    eprintln!(
                                        "Failed to update player leaderboard for guild {}: {}",
                                        guild_id, e
                                    );
                                }
                            }

                            // Update clan leaderboard if configured
                            if guild_data.is_clan_configured() {
                                if let Err(e) = updater
                                    .update_clan_leaderboard(GuildId::new(guild_id))
                                    .await
                                {
                                    eprintln!(
                                        "Failed to update clan leaderboard for guild {}: {}",
                                        guild_id, e
                                    );
                                }
                            }

                            last_update_times.insert(guild_id, now);
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
