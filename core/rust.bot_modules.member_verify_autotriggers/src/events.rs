use silverpelt::ar_event::EventHandlerContext;
use silverpelt::Error;

pub async fn event_listener(ectx: &EventHandlerContext) -> Result<(), Error> {
    let ctx = &ectx.serenity_context;

    match ectx.event {
        silverpelt::ar_event::AntiraidEvent::MemberVerify((user_id, ..)) => {
            // Do something with the user_id TODO
            let Some(rec) = sqlx::query!(
                "SELECT give_roles, remove_roles FROM member_verify_autotriggers__trigger WHERE guild_id = $1",
                ectx.guild_id.to_string()
            )
            .fetch_optional(&ectx.data.pool)
            .await? else {
                return Ok(());
            };

            for role_id in rec.give_roles {
                ctx.http
                    .add_member_role(
                        ectx.guild_id,
                        user_id,
                        role_id.parse::<serenity::all::RoleId>()?,
                        Some("Member verify autotriggers"),
                    )
                    .await?;
            }

            for role_id in rec.remove_roles {
                ctx.http
                    .remove_member_role(
                        ectx.guild_id,
                        user_id,
                        role_id.parse::<serenity::all::RoleId>()?,
                        Some("Member verify autotriggers"),
                    )
                    .await?;
            }
        }
        _ => {} // Ignore all other events
    }

    Ok(())
}
