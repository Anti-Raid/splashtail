/// Web interface access control
#[poise::command(slash_command, subcommands("web_use"))]
pub async fn web(_ctx: silverpelt::Context<'_>) -> Result<(), silverpelt::Error> {
    Ok(())
}

/// This command controls if a user can use the web interface.
#[poise::command(slash_command, rename = "use")]
pub async fn web_use(_ctx: silverpelt::Context<'_>) -> Result<(), silverpelt::Error> {
    Ok(())
}
