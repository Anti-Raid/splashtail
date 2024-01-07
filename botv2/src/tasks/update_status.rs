use rand::seq::SliceRandom;
use serenity::{all::OnlineStatus, gateway::ActivityData};

enum Status {
    Watch,
    Play,
    Listen,
}

pub async fn update_status(
    _pool: &sqlx::PgPool,
    _cache_http: &crate::impls::cache::CacheHttpImpl,
    ctx: &serenity::client::Context,
) -> Result<(), crate::Error> {
    let statuses = vec![
        (Status::Watch, "Development of Anti-Raid v6"),
        (Status::Play, "Development of Anti-Raid v6"),
        (Status::Listen, "Development of Anti-Raid v6"),
    ];

    // Get random status
    let (status, text) = statuses.choose(&mut rand::thread_rng()).unwrap();

    let activity = match status {
        Status::Watch => Some(ActivityData::watching(text.to_string())),
        Status::Play => Some(ActivityData::playing(text.to_string())),
        Status::Listen => Some(ActivityData::listening(text.to_string())),
    };

    ctx.set_presence(activity, OnlineStatus::Online);

    Ok(())
}