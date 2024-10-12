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

/// Verify yourself in this server
#[poise::command(slash_command)]
pub async fn verify(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let template = {
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

    let captcha = captcha_config
        .create_captcha(super::consts::CAPTCHA_CREATE_TIMEOUT)
        .await?;

    msg.edit(
        ctx,
        serenity::all::EditMessage::new()
            .new_attachment(CreateAttachment::bytes(
                captcha.1,
                botox::crypto::gen_random(64) + ".png",
            ))
            .components(vec![serenity::all::CreateActionRow::Buttons(vec![
                serenity::all::CreateButton::new("captcha_verify")
                    .label("Answer")
                    .style(serenity::all::ButtonStyle::Primary),
            ])]),
    )
    .await?;

    let Some(btn_click) = msg
        .await_component_interaction(ctx.serenity_context().shard.clone())
        .author_id(ctx.author().id)
        .timeout(std::time::Duration::from_secs(75))
        .await
    else {
        return Err("You took too long to answer the captcha".into());
    };

    if btn_click.data.custom_id != "captcha_verify" {
        return Err("Invalid button clicked".into());
    }

    // Create modal
    let modal = serenity::all::CreateQuickModal::new("CAPTCHA Verification")
        .timeout(std::time::Duration::from_secs(300))
        .short_field("Answer");

    let Some(response) = btn_click.quick_modal(ctx.serenity_context(), modal).await? else {
        return Err("You took too long to answer the captcha".into());
    };

    if response.inputs[0] != captcha.0 {
        response
            .interaction
            .create_response(
                &ctx.serenity_context().http,
                serenity::all::CreateInteractionResponse::Message(
                    serenity::all::CreateInteractionResponseMessage::new()
                        .content("Invalid captcha answer. Please try verifying again"),
                ),
            )
            .await?;

        return Ok(());
    }

    response
        .interaction
        .create_response(
            &ctx.serenity_context().http,
            serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().content("Verifying..."),
            ),
        )
        .await?;

    // Dispatch MemberVerifyEvent
    silverpelt::ar_event::dispatch_event_to_modules_errflatten(std::sync::Arc::new(
        silverpelt::ar_event::EventHandlerContext {
            guild_id,
            data: ctx.data(),
            event: silverpelt::ar_event::AntiraidEvent::MemberVerify((
                ctx.author().id,
                serde_json::json!({
                    "method": "captcha",
                    "captcha_answer": captcha.0,
                    "captcha_answer_given": response.inputs[0],
                }),
            )),
            serenity_context: ctx.serenity_context().clone(),
        },
    ))
    .await?;

    Ok(())
}
