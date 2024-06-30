use crate::silverpelt::proxysupport::{sandwich::StatusEndpointResponse, ProxyResponse};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Resp {
    ok: bool,
    data: Option<StatusEndpointResponse>,
}

pub async fn sandwich_status_task(ctx: &serenity::client::Context) -> Result<(), crate::Error> {
    let data = ctx.data::<crate::Data>();

    let Some(ref sandwich_url) = crate::config::CONFIG.meta.sandwich_http_api else {
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

    let mut guard = data.proxy_support_data.write().await;

    *guard = Some(ProxyResponse::Sandwich(res).to_support_data());

    Ok(())
}
