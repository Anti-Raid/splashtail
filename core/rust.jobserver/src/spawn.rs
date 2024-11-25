pub async fn spawn_task(
    reqwest_client: &reqwest::Client,
    spawn: &super::Spawn,
) -> Result<super::SpawnResponse, splashcore_rs::Error> {
    let resp = reqwest_client
        .post(format!(
            "{}:{}/spawn",
            config::CONFIG.base_ports.jobserver_base_addr,
            config::CONFIG.base_ports.jobserver
        ))
        .json(spawn)
        .send()
        .await
        .map_err(|e| format!("Failed to initiate task: {}", e))?;

    if resp.status().is_success() {
        Ok(resp.json::<super::SpawnResponse>().await?)
    } else {
        let err_text = resp.text().await?;

        Err(format!("Failed to initiate task: {}", err_text).into())
    }
}
