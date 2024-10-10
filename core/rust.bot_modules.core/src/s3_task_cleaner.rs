use sandwich_driver::{sandwich::StatusEndpointResponse, ProxyResponse};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Resp {
    ok: bool,
    data: Option<StatusEndpointResponse>,
}

pub async fn s3_task_cleaner(
    ctx: &serenity::all::client::Context,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();

    let Some(ref sandwich_url) = config::CONFIG.meta.sandwich_http_api else {
        return Ok(());
    };

    Ok(())
}
