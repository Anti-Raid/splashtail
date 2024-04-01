use serenity::all::FullEvent;
use crate::{Error, silverpelt::EventHandlerContext};

bitflags::bitflags! {
    #[derive(PartialEq)]
    pub struct TriggeredFlags: u32 {
        const NONE = 0;
        const ANTI_INVITE = 1 << 0;
        const ANTI_EVERYONE = 1 << 1;
        const MINIMUM_ACCOUNT_AGE = 1 << 2;
        const MAXIMUM_ACCOUNT_AGE = 1 << 3;
    }
}

pub async fn event_listener(
    ectx: &EventHandlerContext,
) -> Result<(), Error> {
    let ctx = &ectx.serenity_context;
    let event = &ectx.full_event;

    match event {
        FullEvent::Message { new_message } => {
            let data = &ectx.data;
            let config = super::cache::get_config(&data.pool, ectx.guild_id).await?;

            let mut triggered_flags = TriggeredFlags::NONE;

            if config.anti_invite {
                let trimmed_msg = new_message.content.trim().replace("dot", ".").replace("slash", "/").replace(['(', ')'], "");

                if trimmed_msg.contains("discord.gg") || trimmed_msg.contains("discordapp.com/invite") || trimmed_msg.contains("discord.com/invite") {
                    triggered_flags |= TriggeredFlags::ANTI_INVITE;
                }
            }

            if config.anti_everyone && (new_message.content.contains("@everyone") || new_message.mention_everyone()) {
                triggered_flags |= TriggeredFlags::ANTI_EVERYONE;
            }

            if triggered_flags != TriggeredFlags::NONE {
                // For now, don't do anything, punishment support is coming soon
                new_message.delete(&ctx).await?;
            }

            Ok(())
        },
        FullEvent::GuildMemberAddition { new_member } => {
            // Get account creation 
            let data = &ectx.data;
            let config = super::cache::get_config(&data.pool, ectx.guild_id).await?;

            let mut triggered_flags = TriggeredFlags::NONE;

            if let Some(minimum_account_age) = config.minimum_account_age {
                let account_age = new_member.user.created_at();
                let now = chrono::Utc::now();
                if let Some(duration) = chrono::Duration::try_seconds(minimum_account_age) {
                    let diff = now - *account_age;
                    
                    if diff < duration {
                        triggered_flags |= TriggeredFlags::MINIMUM_ACCOUNT_AGE;
                    }    
                }
            }

            if let Some(maximum_account_age) = config.maximum_account_age {
                let account_age = new_member.user.created_at();
                let now = chrono::Utc::now();

                if let Some(duration) = chrono::Duration::try_seconds(maximum_account_age) {
                    let diff = now - *account_age;
                    
                    if diff > duration {
                        triggered_flags |= TriggeredFlags::MAXIMUM_ACCOUNT_AGE;
                    }    
                }
            }

            if triggered_flags != TriggeredFlags::NONE {
                // For now, don't do anything, punishment support is coming soon
                new_member.kick_with_reason(&ctx, "Below configured minimum/maximum account age").await?;
            }

            Ok(())
        },
        _ => Ok(())
    }
}