# Rilistarr

A Discord bot that displays Brawl Stars leaderboards and automatically updates them at configurable intervals.

## Features

- 📊 **Per-Server Leaderboards**: Each Discord server has its own separate leaderboard
- ⏱️ **Auto-Update**: Automatically updates the leaderboard at configurable intervals
- 👑 **First Place Role**: Optionally give a special role to the #1 player

## Setup

1. **Clone and Build**:

   ```bash
   cargo build --release
   ```

2. **Environment Variables**:
   Create a `.env` file based on `.env.example`:

   ```bash
   cp .env.example .env
   ```

   Then fill in your tokens:
   - `DISCORD_TOKEN`: Your Discord bot token from [Discord Developer Portal](https://discord.com/developers/applications)
   - `BRAWL_TOKEN`: Your Brawl Stars API token from [Brawl Stars Developer](https://developer.brawlstars.com/)

3. **Run the Bot**:
   ```bash
   cargo run
   ```

## Commands

### Player Management (Admin or Mod)

- `/player_add <tag>` - Add a player to the leaderboard
- `/player_remove <tag>` - Remove a player from the leaderboard
- `/player_list` - List all players on the leaderboard

### Configuration (Admin only)

- `/config_channel <channel>` - Set the channel for the leaderboard
- `/config_interval <minutes>` - Set the update interval (minimum 5 minutes)
- `/config_role <role>` - Set the role for the #1 player
- `/config_modrole <role>` - Set the mod role (can add/remove players)
- `/config_show` - Show current configuration

### Leaderboard (Admin or Mod)

- `/leaderboard_update` - Force update the leaderboard now
- `/leaderboard_show` - Show the current leaderboard

## Permissions

- **Admin commands**: Only users with the Administrator permission
- **Mod commands**: Users with Administrator permission OR the configured mod role

## License

GPL-3.0
