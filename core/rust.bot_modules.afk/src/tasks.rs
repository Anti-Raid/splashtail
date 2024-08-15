pub async fn afk_task(ctx: &serenity::client::Context) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();
    let pool = &data.pool;

    let mut tx = pool.begin().await?;

    // Check that the last automated vote was one week ago
    let last_task_exec = sqlx::query!(
        "SELECT id FROM last_task_expiry WHERE created_at > NOW() - INTERVAL '1 week' AND task = 'afk' FOR UPDATE",
    )
    .fetch_optional(&mut *tx)
    .await?;

    if last_task_exec.is_some() {
        tx.rollback().await?;
        return Ok(());
    }

    // Lock afk__afks
    sqlx::query!("LOCK TABLE afk__afks IN ACCESS EXCLUSIVE MODE",)
        .execute(&mut *tx)
        .await?;

    // Expire AFKs
    sqlx::query!("DELETE FROM afk__afks WHERE expires_at < NOW()",)
        .execute(&mut *tx)
        .await?;

    // Update last_task_expiry
    sqlx::query!("INSERT INTO last_task_expiry (task) VALUES ('afk')",)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
