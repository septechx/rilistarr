# Rilistarr

A Discord bot that displays Brawl Stars leaderboards and automatically updates them at configurable intervals.

## Features

- 👥 **Clan Leaderboards**: Track and display Brawl Stars clan rankings
- 📊 **Player Leaderboards**: Track and display individual player rankings
- ⏱️ **Auto-Update**: Automatically updates leaderboards at configurable intervals
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

- `/player add <tag>` - Add a player to the leaderboard
- `/player remove <tag>` - Remove a player from the leaderboard
- `/player list` - List all players on the leaderboard

### Clan Management (Admin or Mod)

- `/clan add <tag>` - Add a clan to the leaderboard
- `/clan remove <tag>` - Remove a clan from the leaderboard
- `/clan list` - List all clans on the leaderboard
- `/clan update` - Force update the clan leaderboard

### Configuration (Admin only)

- `/config channel <channel>` - Set the channel for the leaderboard
- `/config interval <minutes>` - Set the update interval (minimum 5 minutes)
- `/config role <role>` - Set the role for the #1 player
- `/config modrole <role>` - Set the mod role (can add/remove players)
- `/config show` - Show current configuration

### Leaderboard (Admin or Mod)

- `/leaderboard update` - Force update the player leaderboard now

## Permissions

- **Admin commands**: Only users with the Administrator permission
- **Mod commands**: Users with Administrator permission OR the configured mod role

## License

GPL-3.0
