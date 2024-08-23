use proxy_support::{sandwich::StatusEndpointResponse, ProxyResponse};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Resp {
    ok: bool,
    data: Option<StatusEndpointResponse>,
}

pub async fn sandwich_status_task(
    ctx: &serenity::all::client::Context,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();

    let Some(ref sandwich_url) = config::CONFIG.meta.sandwich_http_api else {
        return Ok(());
    };

    let res = reqwest::get(&format!("{}/api/status", sandwich_url))
        .await?
        .error_for_status()?
        .json::<Resp>()
        .await?;

    if !res.ok {
        return Err("Sandwich API returned not ok".into());
    }

    let Some(res) = res.data else {
        return Err("No data in response".into());
    };

    let support_data = ProxyResponse::Sandwich(res).to_support_data();

    if support_data.shard_conns.len() > data.props.shard_count().await?.into() {
        return Err("Sandwich API returned more shards than the bot has".into());
    }

    data.props.set_proxysupport_data(support_data).await?;

    Ok(())
}
