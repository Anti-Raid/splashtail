pub async fn afk_task(ctx: &serenity::all::client::Context) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();
    let pool = &data.pool;

    // Expire AFKs
    sqlx::query!("DELETE FROM afk__afks WHERE expires_at < NOW()",)
        .execute(pool)
        .await?;
    Ok(())
}
