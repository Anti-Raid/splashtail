use crate::silverpelt::proxysupport::{
    ProxyResponse,
    sandwich::StatusEndpointResponse
};

pub async fn sandwich_status_task(
    ctx: &serenity::client::Context,
) -> Result<(), crate::Error> {
    let data = ctx.data::<crate::Data>();

    let Some(ref sandwich_url) = crate::config::CONFIG.meta.sandwich_http_api else {
        return Ok(());
    };

    let res = reqwest::get(&format!("{}/api/status", sandwich_url)).await?
        .error_for_status()?
        .json::<StatusEndpointResponse>().await?;

    let mut guard = data.proxy_support_data.write().await;

    *guard = Some(ProxyResponse::Sandwich(res).to_support_data());

    Ok(())
}