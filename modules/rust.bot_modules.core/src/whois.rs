use silverpelt::Context;
use silverpelt::Error;

use serenity::all::{CreateEmbed, Member, Mentionable, User};

fn whois_member<'a>(member: &Member) -> CreateEmbed<'a> {
    let roles = member
        .roles
        .iter()
        .map(|x| x.mention().to_string())
        .collect::<Vec<String>>()
        .join(", ");

    CreateEmbed::default()
        .title("Whois")
        .field("Is Member", "Yes", true)
        .field("User ID", member.user.id.to_string(), true)
        .field(
            "Username",
            {
                if member.user.discriminator.is_some() {
                    member.user.tag()
                } else {
                    member.user.name.to_string()
                }
            },
            true,
        )
        .field(
            "Joined At",
            {
                if let Some(joined_at) = member.joined_at {
                    joined_at.to_string()
                } else {
                    "Unknown".to_string()
                }
            },
            true,
        )
        .field("Created At", member.user.id.created_at().to_string(), true)
        .field("Bot", member.user.bot().to_string(), true)
        .field("Nickname", format!("{:#?}", member.nick), true)
        .field("Roles", roles, true)
}

fn whois_user<'a>(user: &User) -> CreateEmbed<'a> {
    CreateEmbed::default()
        .title("Whois")
        .field("Is Member", "No", true)
        .field("User ID", user.id.to_string(), true)
        .field(
            "Username",
            {
                if user.discriminator.is_some() {
                    user.tag()
                } else {
                    user.name.to_string()
                }
            },
            true,
        )
        .field("Created At", user.id.created_at().to_string(), true)
        .field("Bot", user.bot().to_string(), true)
}

#[poise::command(slash_command)]
pub async fn whois(ctx: Context<'_>, user: Option<User>) -> Result<(), Error> {
    let data = ctx.data();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context());
    let user = user.unwrap_or(ctx.author().clone());

    let embed = {
        if let Some(guild_id) = ctx.guild_id() {
            let member =
                sandwich_driver::member_in_guild(&cache_http, &data.reqwest, guild_id, user.id)
                    .await?;

            if let Some(member) = member {
                whois_member(&member)
            } else {
                whois_user(&user)
            }
        } else {
            whois_user(&user)
        }
    };

    ctx.send(poise::CreateReply::new().embed(embed)).await?;

    Ok(())
}
