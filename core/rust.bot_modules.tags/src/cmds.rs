use serenity::all::AutocompleteChoice;
use silverpelt::{Context, Error};

/// A TagContext is the context for custom tags
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TagContext {
    /// The user that triggered the captcha
    pub user: serenity::all::User,
    /// The guild ID that the user triggered the captcha in
    pub guild_id: serenity::all::GuildId,
    /// The channel ID that the user triggered the captcha in. May be None in some cases (tag not in channel)
    pub channel_id: Option<serenity::all::ChannelId>,
    /// The arguments passed to the tag. Note that it is up to the tag to parse these arguments
    pub args: Option<String>,
}

#[typetag::serde]
impl templating::Context for TagContext {}

/// Tag name autocomplete
async fn tag_name_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<AutocompleteChoice<'a>> {
    let Some(guild_id) = ctx.guild_id() else {
        return Vec::new();
    };

    let data = ctx.data();
    let mut ac = Vec::new();

    let recs = sqlx::query!(
        "SELECT name FROM tags__custom_tags WHERE guild_id = $1 AND name ILIKE $2",
        guild_id.to_string(),
        format!("%{}%", partial)
    )
    .fetch_all(&data.pool)
    .await;

    if let Ok(recs) = recs {
        for rec in recs {
            ac.push(AutocompleteChoice::new(rec.name.clone(), rec.name))
        }
    }

    ac
}

/// Execute a tag
#[poise::command(prefix_command, slash_command)]
pub async fn tag(
    ctx: Context<'_>,
    #[description = "The tag to execute"]
    #[autocomplete = "tag_name_autocomplete"]
    tag: String,
    #[description = "The arguments to pass to the tag"] args: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let template = {
        let tag_template = sqlx::query!(
            "SELECT template FROM tags__custom_tags WHERE guild_id = $1 AND name = $2",
            guild_id.to_string(),
            tag
        )
        .fetch_optional(&ctx.data().pool)
        .await?;

        match tag_template {
            Some(tag_template) => tag_template.template,
            None => {
                return Err("This tag does not exist on this server?".into());
            }
        }
    };

    let mut msg = ctx
        .say("Executing tag... please wait")
        .await?
        .into_message()
        .await?;

    let discord_reply = templating::execute::<_, templating::core::messages::Message>(
        guild_id,
        &template,
        ctx.data().pool.clone(),
        TagContext {
            user: ctx.author().clone(),
            guild_id,
            channel_id: Some(msg.channel_id),
            args,
        },
    )
    .await;

    let discord_reply = match discord_reply {
        Ok(reply) => match templating::core::messages::to_discord_reply(reply) {
            Ok(reply) => reply,
            Err(e) => {
                let embed = serenity::all::CreateEmbed::default()
                    .description(format!("Failed to render tag: {}", e));

                templating::core::messages::DiscordReply {
                    embeds: vec![embed],
                    ..Default::default()
                }
            }
        },
        Err(e) => {
            let embed = serenity::all::CreateEmbed::default()
                .description(format!("Failed to render template: {}", e));

            templating::core::messages::DiscordReply {
                embeds: vec![embed],
                ..Default::default()
            }
        }
    };

    let mut message = serenity::all::EditMessage::default()
        .content("")
        .embeds(discord_reply.embeds);

    if let Some(content) = discord_reply.content {
        message = message.content(content);
    }

    msg.edit(ctx, message).await?;

    Ok(())
}
