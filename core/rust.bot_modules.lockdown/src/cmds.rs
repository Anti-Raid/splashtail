use silverpelt::{Context, Error};

/// Lockdowns
#[poise::command(
    slash_command,
    subcommands(
        "lockdowns_list",
        "lockdowns_tsl",
        "lockdowns_qsl",
        "lockdowns_scl",
        "lockdowns_role",
        "lockdowns_remove"
    )
)]
pub async fn lockdowns(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Lists all currently ongoing lockdowns in summary form
#[poise::command(slash_command, guild_only, rename = "list")]
pub async fn lockdowns_list(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let lockdowns = sqlx::query!(
        "SELECT id, type, reason FROM lockdown__guild_lockdowns WHERE guild_id = $1",
        guild_id.to_string()
    )
    .fetch_all(&ctx.data().pool)
    .await?;

    if lockdowns.is_empty() {
        return Err("No active lockdowns".into());
    }

    let mut msg = String::new();

    for lockdown in lockdowns {
        msg.push_str(&format!(
            "ID: {}, Type: {}, Reason: {}\n",
            lockdown.id, lockdown.r#type, lockdown.reason
        ));
    }

    ctx.send(
        poise::CreateReply::new().embed(
            serenity::all::CreateEmbed::new()
                .title("Active Lockdowns")
                .description(msg),
        ),
    )
    .await?;

    Ok(())
}

/// Starts a traditional server lockdown
#[poise::command(slash_command, guild_only, rename = "tsl")]
pub async fn lockdowns_tsl(ctx: Context<'_>, reason: String) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let data = ctx.data();

    // Get the current lockdown set
    let mut lockdowns = lockdowns::LockdownSet::guild(guild_id, &data.pool)
        .await
        .map_err(|e| format!("Error while fetching lockdown set: {}", e))?;

    // Create the lockdown
    let lockdown_type = lockdowns::tsl::TraditionalServerLockdown {};

    let lockdown_data = lockdowns::LockdownData {
        cache_http: botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context()),
        pool: data.pool.clone(),
        reqwest: data.reqwest.clone(),
        object_store: data.object_store.clone(),
    };

    ctx.defer().await?;

    lockdowns
        .easy_apply(Box::new(lockdown_type), &lockdown_data, &reason)
        .await
        .map_err(|e| format!("Error while applying lockdown: {}", e))?;

    ctx.say("Lockdown started").await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, rename = "qsl")]
/// Starts a quick server lockdown
pub async fn lockdowns_qsl(ctx: Context<'_>, reason: String) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let data = ctx.data();

    // Get the current lockdown set
    let mut lockdowns = lockdowns::LockdownSet::guild(guild_id, &data.pool)
        .await
        .map_err(|e| format!("Error while fetching lockdown set: {}", e))?;

    // Create the lockdown
    let lockdown_type = lockdowns::qsl::QuickServerLockdown {};

    let lockdown_data = lockdowns::LockdownData {
        cache_http: botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context()),
        pool: data.pool.clone(),
        reqwest: data.reqwest.clone(),
        object_store: data.object_store.clone(),
    };

    ctx.defer().await?;

    lockdowns
        .easy_apply(Box::new(lockdown_type), &lockdown_data, &reason)
        .await
        .map_err(|e| format!("Error while applying lockdown: {}", e))?;

    ctx.say("Lockdown started").await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, rename = "scl")]
/// Starts a single channel lockdown
pub async fn lockdowns_scl(
    ctx: Context<'_>,
    channel: Option<serenity::all::ChannelId>,
    reason: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let data = ctx.data();
    let channel = channel.unwrap_or(ctx.channel_id());

    // Get the current lockdown set
    let mut lockdowns = lockdowns::LockdownSet::guild(guild_id, &data.pool)
        .await
        .map_err(|e| format!("Error while fetching lockdown set: {}", e))?;

    // Create the lockdown
    let lockdown_type = lockdowns::scl::SingleChannelLockdown(channel);

    let lockdown_data = lockdowns::LockdownData {
        cache_http: botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context()),
        pool: data.pool.clone(),
        reqwest: data.reqwest.clone(),
        object_store: data.object_store.clone(),
    };

    ctx.defer().await?;

    lockdowns
        .easy_apply(Box::new(lockdown_type), &lockdown_data, &reason)
        .await
        .map_err(|e| format!("Error while applying lockdown: {}", e))?;

    ctx.say("Lockdown started").await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, rename = "role")]
/// Starts a single channel lockdown
pub async fn lockdowns_role(
    ctx: Context<'_>,
    role: serenity::all::RoleId,
    reason: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let data = ctx.data();

    // Get the current lockdown set
    let mut lockdowns = lockdowns::LockdownSet::guild(guild_id, &data.pool)
        .await
        .map_err(|e| format!("Error while fetching lockdown set: {}", e))?;

    // Create the lockdown
    let lockdown_type = lockdowns::role::RoleLockdown(role);

    let lockdown_data = lockdowns::LockdownData {
        cache_http: botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context()),
        pool: data.pool.clone(),
        reqwest: data.reqwest.clone(),
        object_store: data.object_store.clone(),
    };

    ctx.defer().await?;

    lockdowns
        .easy_apply(Box::new(lockdown_type), &lockdown_data, &reason)
        .await
        .map_err(|e| format!("Error while applying lockdown: {}", e))?;

    ctx.say("Lockdown started").await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, rename = "remove")]
/// Remove a lockdown by ID
pub async fn lockdowns_remove(ctx: Context<'_>, id: String) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let data = ctx.data();

    // Get the current lockdown set
    let mut lockdowns = lockdowns::LockdownSet::guild(guild_id, &data.pool)
        .await
        .map_err(|e| format!("Error while fetching lockdown set: {}", e))?;

    let lockdown_data = lockdowns::LockdownData {
        cache_http: botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context()),
        pool: data.pool.clone(),
        reqwest: data.reqwest.clone(),
        object_store: data.object_store.clone(),
    };

    ctx.defer().await?;

    lockdowns
        .easy_remove(id.parse()?, &lockdown_data)
        .await
        .map_err(|e| format!("Error while applying lockdown: {}", e))?;

    ctx.say("Lockdown removed").await?;

    Ok(())
}
