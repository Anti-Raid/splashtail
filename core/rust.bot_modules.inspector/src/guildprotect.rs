/// Since discord does not store past guild icons for a guild, so we have to store it manually
/// on s3
pub async fn fetch_guild_icon(
    data: &silverpelt::data::Data,
    guild_id: serenity::all::GuildId,
) -> Result<Vec<u8>, silverpelt::Error> {
    let url = data.object_store.get_url(
        &format!("inspector/guild_icons/{}", guild_id),
        std::time::Duration::from_secs(120),
    );

    let response = reqwest::get(url).await?.error_for_status()?;

    Ok(response.bytes().await?.to_vec())
}

/// Saves a guild icon to s3
pub async fn save_guild_icon(
    reqwest_client: &reqwest::Client,
    object_store: &splashcore_rs::objectstore::ObjectStore,
    guild_id: serenity::all::GuildId,
    icon: &[u8],
) -> Result<(), silverpelt::Error> {
    object_store
        .upload_file(
            reqwest_client,
            &format!("inspector/guild_icons/{}", guild_id),
            icon,
        )
        .await?;

    Ok(())
}

/// A snapshot stores the state of a guild at a certain point in time for inspector guild protection
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
    pub async fn save(
        &self,
        pool: &sqlx::PgPool,
        reqwest_client: &reqwest::Client,
        object_store: &splashcore_rs::objectstore::ObjectStore,
    ) -> Result<(), silverpelt::Error> {
        // Download icon from discord
        let mut icon_bytes = None;
        if let Some(icon) = self.icon_url() {
            let bytes = reqwest_client
                .get(icon)
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;

            icon_bytes = Some(bytes.to_vec());
        }

        let mut tx = pool.begin().await?;

        sqlx::query!(
            "INSERT INTO inspector__guilds (guild_id, name, icon) VALUES ($1, $2, $3) ON CONFLICT (guild_id) DO UPDATE SET name = $2, icon = $3",
            self.guild_id.to_string(),
            &self.name.to_string(),
            self.icon.as_ref().map(|x| x.to_string()),
        )
        .execute(&mut *tx)
        .await?;

        if let Some(icon_bytes) = icon_bytes {
            save_guild_icon(reqwest_client, object_store, self.guild_id, &icon_bytes).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Reverts a guild to the snapshot
    pub async fn revert(
        &self,
        ctx: &serenity::all::Context,
        data: &silverpelt::data::Data,
        change_name: bool,
        change_icon: bool,
    ) -> Result<(), silverpelt::Error> {
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

            silverpelt::ar_event::dispatch_event_to_modules_errflatten(
                std::sync::Arc::new(silverpelt::ar_event::EventHandlerContext {
                    guild_id: self.guild_id,
                    data: data.clone().into(),
                    event: silverpelt::ar_event::AntiraidEvent::Custom(
                        Box::new(std_events::auditlog::AuditLogDispatchEvent {
                            event_name: "AR/Inspector_GuildProtectRevert".to_string(),
                            event_titlename: "(Anti-Raid) Guild Protection: Revert Changes".to_string(),
                            event_data: indexmap::indexmap! {
                                "name".to_string() => self.name.clone().into(),
                                "triggered_flags".to_string() => triggered_protections.iter_names().map(|(flag, _)| flag.to_string()).collect::<Vec<String>>().join(", ").into(),
                            }
                        })
                    ),
                    serenity_context: ctx.clone(),
                }),
            )
            .await?;
        }

        Ok(())
    }
}
