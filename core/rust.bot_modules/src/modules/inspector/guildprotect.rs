use super::types::GuildProtectionOptions;

/// Since discord does not store past guild icons for a guild, so we have to store it manually
/// on s3
pub async fn fetch_guild_icon(
    data: &crate::Data,
    guild_id: serenity::model::id::GuildId,
) -> Result<Vec<u8>, crate::Error> {
    let url = data.object_store.get_url(
        &format!("inspector/guild_icons/{}", guild_id),
        std::time::Duration::from_secs(120),
    );

    let response = reqwest::get(url).await?.error_for_status()?;

    Ok(response.bytes().await?.to_vec())
}

/// Saves a guild icon to s3
pub async fn save_guild_icon(
    data: &crate::Data,
    guild_id: serenity::model::id::GuildId,
    icon: &[u8],
) -> Result<(), crate::Error> {
    data.object_store
        .upload_file(
            &data.reqwest,
            &format!("inspector/guild_icons/{}", guild_id),
            icon,
        )
        .await?;

    Ok(())
}

pub async fn save_all_guilds_initial(
    ctx: serenity::all::Context,
    data: &crate::Data,
) -> Result<(), crate::Error> {
    // For every guild with inspector enabled, check if the guild is saved in inspector__guilds, if not save
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(&ctx);
    let reqwest_client = &data.reqwest;
    let pool = &data.pool;

    bitflags::bitflags! {
        #[derive(PartialEq, Debug, Clone, Copy)]
        pub struct InitialProtectionTriggers: i32 {
            const NONE = 0;
            const NAME = 1 << 1;
            const ICON = 1 << 2;
        }
    }

    for guild_id in ctx.cache.guilds() {
        // Ensure shard id
        let shard_id = serenity::utils::shard_id(guild_id, data.props.shard_count().try_into()?);

        if ctx.shard_id.0 != shard_id {
            continue;
        }

        let module_enabled =
            match crate::silverpelt::module_config::is_module_enabled(pool, guild_id, "inspector")
                .await
            {
                Ok(enabled) => enabled,
                Err(e) => {
                    log::error!("Error while checking if module is enabled: {}", e);
                    continue;
                }
            };

        if !module_enabled {
            continue;
        }

        // Fetch the config
        let config = match super::cache::get_config(pool, guild_id).await {
            Ok(config) => config,
            Err(e) => {
                log::error!("Error while fetching config: {}", e);
                continue;
            }
        };

        if config
            .guild_protection
            .contains(GuildProtectionOptions::DISABLED)
        {
            continue;
        }

        // We anyways need to fetch the guild anyways, so do that
        let guild = match proxy_support::guild(&cache_http, reqwest_client, guild_id).await {
            Ok(guild) => guild,
            Err(e) => {
                log::error!("Error while fetching guild: {}", e);
                continue;
            }
        };

        let guild_row = match sqlx::query!(
            "SELECT name, icon FROM inspector__guilds WHERE guild_id = $1",
            guild_id.to_string(),
        )
        .fetch_optional(pool)
        .await
        {
            Ok(row) => row,
            Err(e) => {
                log::error!("Error while fetching guild row: {}", e);
                continue;
            }
        };

        if let Some(guild_row) = guild_row {
            match (Snapshot {
                guild_id,
                name: guild_row.name.clone(),
                icon: guild_row.icon.clone(),
            }
            .revert(&ctx, data, guild_row.name != guild.name, {
                if guild_row.icon.is_some() {
                    guild_row.icon != guild.icon.map(|x| x.to_string())
                } else {
                    false
                }
            }))
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error while reverting guild: {}", e);
                    continue;
                }
            }
        } else {
            // Guild not saved, save it
            match (Snapshot {
                guild_id,
                name: guild.name.to_string(),
                icon: guild.icon.map(|x| x.to_string()),
            })
            .save(data)
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error while saving guild: {}", e);
                    continue;
                }
            }
        }
    }

    Ok(())
}

pub struct Snapshot {
    pub guild_id: serenity::all::GuildId,
    pub name: String,
    pub icon: Option<String>,
}

impl Snapshot {
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|x| {
            format!(
                "https://cdn.discordapp.com/icons/{}/{}.webp",
                self.guild_id, x
            )
        })
    }

    /// Saves a new snapshot of a guild to the database
    pub async fn save(&self, data: &crate::Data) -> Result<(), crate::Error> {
        // Download icon from discord
        let mut icon_bytes = None;
        if let Some(icon) = self.icon_url() {
            let bytes = reqwest::get(icon)
                .await?
                .error_for_status()?
                .bytes()
                .await?;

            icon_bytes = Some(bytes.to_vec());
        }

        let mut tx = data.pool.begin().await?;

        sqlx::query!(
        "INSERT INTO inspector__guilds (guild_id, name, icon) VALUES ($1, $2, $3) ON CONFLICT (guild_id) DO UPDATE SET name = $2, icon = $3",
        self.guild_id.to_string(),
        &self.name.to_string(),
        self.icon.as_ref().map(|x| x.to_string()),
    )
    .execute(&mut *tx)
    .await?;

        if let Some(icon_bytes) = icon_bytes {
            save_guild_icon(data, self.guild_id, &icon_bytes).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Reverts a guild to the snapshot
    pub async fn revert(
        &self,
        ctx: &serenity::all::Context,
        data: &crate::Data,
        change_name: bool,
        change_icon: bool,
    ) -> Result<(), crate::Error> {
        bitflags::bitflags! {
            #[derive(PartialEq, Debug, Clone, Copy)]
            pub struct InitialProtectionTriggers: i32 {
                const NONE = 0;
                const NAME = 1 << 1;
                const ICON = 1 << 2;
            }
        }

        // Check if theres any changes we need to revert
        let mut edit_guild = serenity::all::EditGuild::new();
        let mut triggered_protections = InitialProtectionTriggers::NONE;
        if change_name {
            // Name changed, we need to revert the change on the guild as the bot was down when it occurred
            edit_guild = edit_guild.name(&self.name);
            triggered_protections |= InitialProtectionTriggers::NAME;
        }

        if let Some(ref icon) = self.icon {
            if change_icon {
                // Icon changed, we need to revert the change on the guild as the bot was down when it occurred
                if let Ok(icon_data) = fetch_guild_icon(data, self.guild_id).await {
                    edit_guild = edit_guild.icon(Some(&serenity::all::CreateAttachment::bytes(
                        icon_data,
                        if icon.starts_with("a_") {
                            "image.gif"
                        } else {
                            "image.png"
                        },
                    )));
                    triggered_protections |= InitialProtectionTriggers::ICON;
                }
            }
        }

        if triggered_protections != InitialProtectionTriggers::NONE {
            let mut tg = vec![];

            for (flag, _) in triggered_protections.iter_names() {
                tg.push(flag.to_string());
            }

            let reason = format!("Reverting guild changes: {}", tg.join(", "));

            edit_guild = edit_guild.audit_log_reason(&reason);
            match self.guild_id.edit(&ctx.http, edit_guild).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(format!("Error while reverting guild changes: {}", e).into());
                }
            }

            // Create audit log
            // Send audit logs if Audit Logs module is enabled
            if crate::silverpelt::module_config::is_module_enabled(
                &data.pool,
                self.guild_id,
                "auditlogs",
            )
            .await?
            {
                let imap = indexmap::indexmap! {
                    "name".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: self.name.clone().into() },
                    "triggered_flags".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: triggered_protections.iter_names().map(|(flag, _)| flag.to_string()).collect::<Vec<String>>().join(", ").into() },
                };

                crate::modules::auditlogs::events::dispatch_audit_log(
                    ctx,
                    "AR/GuildProtectRevert",
                    "(Anti-Raid) Guild Protection: Revert Changes",
                    imap,
                    self.guild_id,
                )
                .await?;
            }
        }

        Ok(())
    }
}
