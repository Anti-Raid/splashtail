use sandwich_driver::GetStatusResponse;
use tokio::sync::RwLock;

pub static SANDWICH_STATUS: std::sync::LazyLock<RwLock<Option<GetStatusResponse>>> =
    std::sync::LazyLock::new(|| RwLock::new(None));

pub async fn sandwich_status_task(
    ctx: &serenity::all::client::Context,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();

    let mut sandwich_status_guard = SANDWICH_STATUS.write().await;

    let status = sandwich_driver::get_status(&data.reqwest).await?;

    if status.shard_conns.len() > data.props.shard_count().await?.into() {
        return Err("Sandwich API returned more shards than the bot has".into());
    }

    *sandwich_status_guard = Some(status);

    Ok(())
}
