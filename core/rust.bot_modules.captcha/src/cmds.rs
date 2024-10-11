use serenity::all::CreateAttachment;
use silverpelt::Context;
use silverpelt::Error;

const TEST_CAPTCHA_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// Test the captcha system
#[poise::command(slash_command)]
pub async fn captcha_test(ctx: Context<'_>, use_sample: Option<bool>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let template = {
        if !use_sample.unwrap_or(false) {
            let captcha_template = sqlx::query!(
                "SELECT template FROM captcha__guild_captchas WHERE guild_id = $1",
                guild_id.to_string()
            )
            .fetch_optional(&ctx.data().pool)
            .await?;

            match captcha_template {
                Some(captcha_template) => captcha_template.template,
                None => {
                    return Err("This guild does not have a captcha template set".into());
                }
            }
        } else {
            include_str!("templates/sample.art").to_string()
        }
    };

    let mut msg = ctx
        .say("Creating captcha, please wait...")
        .await?
        .into_message()
        .await?;

    let captcha_config = templating::execute::<_, super::templater::CaptchaConfig>(
        guild_id,
        &template,
        ctx.data().pool.clone(),
        super::templater::CaptchaContext {
            guild_id,
            user_id: ctx.author().id,
            channel_id: Some(msg.channel_id),
        },
    )
    .await?;

    let captcha = captcha_config.create_captcha(TEST_CAPTCHA_TIMEOUT).await?;

    msg.edit(
        ctx,
        serenity::all::EditMessage::new()
            .content(format!("Answer: {}", captcha.0))
            .new_attachment(CreateAttachment::bytes(
                captcha.1,
                botox::crypto::gen_random(64) + ".png",
            )),
    )
    .await?;

    Ok(())
}
