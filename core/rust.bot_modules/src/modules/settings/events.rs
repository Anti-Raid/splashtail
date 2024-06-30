use crate::silverpelt::silverpelt_cache::SILVERPELT_CACHE;
use crate::{silverpelt::EventHandlerContext, Error};
use serenity::all::FullEvent;

pub async fn event_listener(ectx: &EventHandlerContext) -> Result<(), Error> {
    let event = &ectx.full_event;

    // If the user changes their role or if it cant be determined if they have
    // yet they've changed somethine, remove the resolved perms cache
    #[allow(clippy::single_match)] // May be more cases in the future
    match event {
        FullEvent::GuildMemberUpdate {
            old_if_available,
            new,
            event,
        } => {
            let mut remove_cache = true;

            if let Some(old) = old_if_available {
                if let Some(new) = new {
                    if old.roles == new.roles {
                        remove_cache = false;
                    }
                }
            }

            if remove_cache {
                let guild_id = ectx.guild_id;
                let user_id = event.user.id;
                if let Err(err) = SILVERPELT_CACHE
                    .command_permission_cache
                    .invalidate_entries_if(move |k, _| k.0 == guild_id && k.1 == user_id)
                {
                    log::error!(
                        "Failed to invalidate command permission cache for guild {}: {}",
                        guild_id,
                        err
                    );
                } else {
                    log::debug!(
                        "Invalidated cache for guild {} and user {}",
                        guild_id,
                        user_id
                    );
                }
            }
        }
        _ => {}
    }

    Ok(())
}
