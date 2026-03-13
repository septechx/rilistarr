use crate::data::{DataError, GuildData};
use crate::leaderboard::{LeaderboardError, LeaderboardUpdater};
use crate::permissions::{has_admin_or_mod_permission, has_admin_permission};
use poise::serenity_prelude as serenity;
use poise::Command;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

type Data = ();

/// Show this help menu
#[poise::command(slash_command, prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Add a player to the leaderboard (Admin or Mod only)
#[poise::command(slash_command, guild_only, rename = "player add")]
pub async fn player_add(
    ctx: Context<'_>,
    #[description = "Player tag"]
    #[min_length = 3]
    tag: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let member = ctx
        .author_member()
        .await
        .ok_or("Could not get member info")?;

    // Check permissions
    if !has_admin_or_mod_permission(ctx.serenity_context(), guild_id, &member).await? {
        ctx.say("❌ You need to be an admin or have the mod role to use this command.")
            .await?;
        return Ok(());
    }

    let mut guild_data = GuildData::load(guild_id.get())?;

    match guild_data.add_player(tag.clone()) {
        Ok(_) => {
            guild_data.save(guild_id.get())?;
            ctx.say(format!("✅ Player `{}` added to the leaderboard!", tag))
                .await?;
        }
        Err(DataError::PlayerAlreadyExists) => {
            ctx.say(format!(
                "⚠️ Player `{}` is already on the leaderboard.",
                tag
            ))
            .await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Error adding player: {}", e)).await?;
        }
    }

    Ok(())
}

/// Remove a player from the leaderboard (Admin or Mod only)
#[poise::command(slash_command, guild_only, rename = "player remove")]
pub async fn player_remove(
    ctx: Context<'_>,
    #[description = "Player tag"] tag: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let member = ctx
        .author_member()
        .await
        .ok_or("Could not get member info")?;

    if !has_admin_or_mod_permission(ctx.serenity_context(), guild_id, &member).await? {
        ctx.say("❌ You need to be an admin or have the mod role to use this command.")
            .await?;
        return Ok(());
    }

    let mut guild_data = GuildData::load(guild_id.get())?;

    match guild_data.remove_player(&tag) {
        Ok(_) => {
            guild_data.save(guild_id.get())?;
            ctx.say(format!("✅ Player `{}` removed from the leaderboard!", tag))
                .await?;
        }
        Err(DataError::PlayerNotFound) => {
            ctx.say(format!("⚠️ Player `{}` is not on the leaderboard.", tag))
                .await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Error removing player: {}", e)).await?;
        }
    }

    Ok(())
}

/// List all players on the leaderboard
#[poise::command(slash_command, guild_only, rename = "player list")]
pub async fn player_list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let guild_data = GuildData::load(guild_id.get())?;

    if guild_data.players.is_empty() {
        ctx.say("📋 No players on the leaderboard yet. Use `/player add` to add some!")
            .await?;
        return Ok(());
    }

    let players_list = guild_data.players.join("\n");
    ctx.say(format!(
        "📋 **Players on leaderboard:**\n```{}```",
        players_list
    ))
    .await?;

    Ok(())
}

/// Set the leaderboard channel (Admin only)
#[poise::command(slash_command, guild_only, rename = "config channel")]
pub async fn config_channel(
    ctx: Context<'_>,
    #[description = "Channel for the leaderboard"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let member = ctx
        .author_member()
        .await
        .ok_or("Could not get member info")?;

    if !has_admin_permission(&member) {
        ctx.say("❌ You need to be an admin to use this command.")
            .await?;
        return Ok(());
    }

    let channel_id = channel.id;
    let mut guild_data = GuildData::load(guild_id.get())?;
    guild_data.set_channel(channel_id.get());
    guild_data.save(guild_id.get())?;

    ctx.say(format!("✅ Leaderboard channel set to <#{}>", channel_id))
        .await?;

    Ok(())
}

/// Set the update interval in minutes (Admin only)
#[poise::command(slash_command, guild_only, rename = "config interval")]
pub async fn config_interval(
    ctx: Context<'_>,
    #[description = "Update interval in minutes (minimum 5)"]
    #[min = 5]
    minutes: u64,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let member = ctx
        .author_member()
        .await
        .ok_or("Could not get member info")?;

    if !has_admin_permission(&member) {
        ctx.say("❌ You need to be an admin to use this command.")
            .await?;
        return Ok(());
    }

    let mut guild_data = GuildData::load(guild_id.get())?;
    guild_data.set_interval(minutes);
    guild_data.save(guild_id.get())?;

    ctx.say(format!("✅ Update interval set to {} minutes.", minutes))
        .await?;

    Ok(())
}

/// Set the role for #1 player (Admin only)
#[poise::command(slash_command, guild_only, rename = "config role")]
pub async fn config_role(
    ctx: Context<'_>,
    #[description = "Role for the #1 player"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let member = ctx
        .author_member()
        .await
        .ok_or("Could not get member info")?;

    if !has_admin_permission(&member) {
        ctx.say("❌ You need to be an admin to use this command.")
            .await?;
        return Ok(());
    }

    let mut guild_data = GuildData::load(guild_id.get())?;
    guild_data.set_first_place_role(role.id.get());
    guild_data.save(guild_id.get())?;

    ctx.say(format!("✅ First place role set to {}", role.name))
        .await?;

    Ok(())
}

/// Set the mod role for player management (Admin only)
#[poise::command(slash_command, guild_only, rename = "config modrole")]
pub async fn config_modrole(
    ctx: Context<'_>,
    #[description = "Role for moderators (can add/remove players)"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let member = ctx
        .author_member()
        .await
        .ok_or("Could not get member info")?;

    if !has_admin_permission(&member) {
        ctx.say("❌ You need to be an admin to use this command.")
            .await?;
        return Ok(());
    }

    let mut guild_data = GuildData::load(guild_id.get())?;
    guild_data.set_mod_role(role.id.get());
    guild_data.save(guild_id.get())?;

    ctx.say(format!("✅ Mod role set to {}", role.name)).await?;

    Ok(())
}

/// Show current configuration
#[poise::command(slash_command, guild_only, rename = "config show")]
pub async fn config_show(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let guild_data = GuildData::load(guild_id.get())?;

    let channel_str = guild_data
        .leaderboard_channel_id
        .map(|id| format!("<#{}>", id))
        .unwrap_or_else(|| "Not set".to_string());

    let role_str = guild_data
        .first_place_role_id
        .map(|id| format!("<@&{}>", id))
        .unwrap_or_else(|| "Not set".to_string());

    let mod_role_str = guild_data
        .mod_role_id
        .map(|id| format!("<@&{}>", id))
        .unwrap_or_else(|| "Not set".to_string());

    let message_str = guild_data
        .leaderboard_message_id
        .map(|id| format!("{}", id))
        .unwrap_or_else(|| "Not created yet".to_string());

    let config_text = format!(
        "⚙️ **Current Configuration**\n\n📢 Leaderboard Channel: {}\n📝 Leaderboard Message ID: {}\n⏱️ Update Interval: {} minutes\n👑 First Place Role: {}\n🛡️ Mod Role: {}\n👥 Players: {}",
        channel_str,
        message_str,
        guild_data.update_interval_minutes,
        role_str,
        mod_role_str,
        guild_data.players.len()
    );

    ctx.say(config_text).await?;

    Ok(())
}

/// Force update the leaderboard now (Admin or Mod only)
#[poise::command(slash_command, guild_only, rename = "leaderboard update")]
pub async fn leaderboard_update(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let member = ctx
        .author_member()
        .await
        .ok_or("Could not get member info")?;

    if !has_admin_or_mod_permission(ctx.serenity_context(), guild_id, &member).await? {
        ctx.say("❌ You need to be an admin or have the mod role to use this command.")
            .await?;
        return Ok(());
    }

    let token = std::env::var("BRAWL_TOKEN").map_err(|_| "BRAWL_TOKEN not set")?;

    ctx.defer().await?;

    let updater = LeaderboardUpdater::new(ctx.serenity_context().clone(), &token);

    match updater.update_player_leaderboard(guild_id).await {
        Ok(_) => {
            ctx.say("✅ Leaderboard updated successfully!").await?;
        }
        Err(LeaderboardError::ChannelNotConfigured) => {
            ctx.say("❌ Leaderboard channel not configured. Use `/config channel` first.")
                .await?;
        }
        Err(LeaderboardError::NoPlayers) => {
            ctx.say("❌ No players on the leaderboard. Use `/player add` to add some.")
                .await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Error updating leaderboard: {}", e))
                .await?;
        }
    }

    Ok(())
}

/// Add a clan to the leaderboard (Admin or Mod only)
#[poise::command(slash_command, guild_only, rename = "clan add")]
pub async fn clan_add(
    ctx: Context<'_>,
    #[description = "Clan tag"]
    #[min_length = 3]
    tag: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let member = ctx
        .author_member()
        .await
        .ok_or("Could not get member info")?;

    if !has_admin_or_mod_permission(ctx.serenity_context(), guild_id, &member).await? {
        ctx.say("❌ You need to be an admin or have the mod role to use this command.")
            .await?;
        return Ok(());
    }

    let mut guild_data = GuildData::load(guild_id.get())?;

    match guild_data.add_clan(tag.clone()) {
        Ok(_) => {
            guild_data.save(guild_id.get())?;
            ctx.say(format!("✅ Clan `{}` added to the leaderboard!", tag))
                .await?;
        }
        Err(DataError::ClanAlreadyExists) => {
            ctx.say(format!("⚠️ Clan `{}` is already on the leaderboard.", tag))
                .await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Error adding clan: {}", e)).await?;
        }
    }

    Ok(())
}

/// Remove a clan from the leaderboard (Admin or Mod only)
#[poise::command(slash_command, guild_only, rename = "clan remove")]
pub async fn clan_remove(
    ctx: Context<'_>,
    #[description = "Clan tag"] tag: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let member = ctx
        .author_member()
        .await
        .ok_or("Could not get member info")?;

    if !has_admin_or_mod_permission(ctx.serenity_context(), guild_id, &member).await? {
        ctx.say("❌ You need to be an admin or have the mod role to use this command.")
            .await?;
        return Ok(());
    }

    let mut guild_data = GuildData::load(guild_id.get())?;

    match guild_data.remove_clan(&tag) {
        Ok(_) => {
            guild_data.save(guild_id.get())?;
            ctx.say(format!("✅ Clan `{}` removed from the leaderboard!", tag))
                .await?;
        }
        Err(DataError::ClanNotFound) => {
            ctx.say(format!("⚠️ Clan `{}` is not on the leaderboard.", tag))
                .await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Error removing clan: {}", e)).await?;
        }
    }

    Ok(())
}

/// List all clans on the leaderboard
#[poise::command(slash_command, guild_only, rename = "clan list")]
pub async fn clan_list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let guild_data = GuildData::load(guild_id.get())?;

    if guild_data.clans.is_empty() {
        ctx.say("📋 No clans on the leaderboard yet. Use `/clan add` to add some!")
            .await?;
        return Ok(());
    }

    let clans_list = guild_data.clans.join("\n");
    ctx.say(format!(
        "📋 **Clans on leaderboard:**\n```{}```",
        clans_list
    ))
    .await?;

    Ok(())
}

/// Force update the clan leaderboard now (Admin or Mod only)
#[poise::command(slash_command, guild_only, rename = "clan update")]
pub async fn clan_update(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be used in a server")?;
    let member = ctx
        .author_member()
        .await
        .ok_or("Could not get member info")?;

    if !has_admin_or_mod_permission(ctx.serenity_context(), guild_id, &member).await? {
        ctx.say("❌ You need to be an admin or have the mod role to use this command.")
            .await?;
        return Ok(());
    }

    let token = std::env::var("BRAWL_TOKEN").map_err(|_| "BRAWL_TOKEN not set")?;

    ctx.defer().await?;

    let updater = LeaderboardUpdater::new(ctx.serenity_context().clone(), &token);

    match updater.update_clan_leaderboard(guild_id).await {
        Ok(_) => {
            ctx.say("✅ Clan leaderboard updated successfully!").await?;
        }
        Err(LeaderboardError::ChannelNotConfigured) => {
            ctx.say("❌ Leaderboard channel not configured. Use `/config channel` first.")
                .await?;
        }
        Err(LeaderboardError::NoClans) => {
            ctx.say("❌ No clans on the leaderboard. Use `/clan add` to add some.")
                .await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Error updating clan leaderboard: {}", e))
                .await?;
        }
    }

    Ok(())
}

pub fn get_commands() -> Vec<Command<(), Error>> {
    vec![
        help(),
        player_add(),
        player_remove(),
        player_list(),
        config_channel(),
        config_interval(),
        config_role(),
        config_modrole(),
        config_show(),
        leaderboard_update(),
        clan_add(),
        clan_remove(),
        clan_list(),
        clan_update(),
    ]
}
