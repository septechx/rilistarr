use crate::data::GuildData;
use serenity::model::prelude::*;
use serenity::prelude::*;

/// Check if a user has admin permissions or the configured mod role
pub async fn has_admin_or_mod_permission(
    _ctx: &Context,
    guild_id: GuildId,
    member: &Member,
) -> Result<bool, serenity::Error> {
    // Check for administrator permission first
    if let Some(permissions) = member.permissions {
        if permissions.administrator() {
            return Ok(true);
        }
    }

    // Check for configured mod role
    if let Ok(guild_data) = GuildData::load(guild_id.get()) {
        if let Some(mod_role_id) = guild_data.mod_role_id {
            let mod_role_id = RoleId::new(mod_role_id);
            if member.roles.contains(&mod_role_id) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Check if a user has admin permissions (for config commands)
pub fn has_admin_permission(member: &Member) -> bool {
    member
        .permissions
        .map(|p| p.administrator())
        .unwrap_or(false)
}
