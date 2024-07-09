use proxy_support::{sandwich::StatusEndpointResponse, ProxyResponse};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Resp {
    ok: bool,
    data: Option<StatusEndpointResponse>,
}

pub async fn sandwich_status_task(ctx: &serenity::client::Context) -> Result<(), crate::Error> {
    let data = ctx.data::<crate::Data>();

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

    if support_data.shard_conns.len() > data.props.shard_count().into() {
        // TODO: Restart instead of panic
        panic!("Sandwich returned more shard groups than we have shards, aborting to ensure re-sharding");
    }

    let mut guard = data.proxy_support_data.write().await;

    *guard = Some(support_data);

    Ok(())
}
