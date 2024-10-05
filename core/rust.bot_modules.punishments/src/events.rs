use silverpelt::ar_event::{AntiraidEvent, EventHandlerContext};

/// Punishment autotrigger event listener
pub(crate) async fn event_listener(ectx: &EventHandlerContext) -> Result<(), silverpelt::Error> {
    match ectx.event {
        AntiraidEvent::StingCreate(ref sting) => {
            let guild_id = sting.guild_id;

            crate::core::autotrigger(&ectx.serenity_context, guild_id).await
        }
        _ => {
            return Ok(()); // Ignore non-discord events
        }
    }
}
