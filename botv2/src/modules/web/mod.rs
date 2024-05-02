use indexmap::indexmap;

/// Web interface access control
#[poise::command(prefix_command, slash_command, subcommands("web_use"))]
pub async fn web(_ctx: crate::Context<'_>) -> Result<(), crate::Error> {
    Ok(())
}

/// This command controls if a user can use the web interface.
#[poise::command(prefix_command, slash_command, rename = "use")]
pub async fn web_use(_ctx: crate::Context<'_>) -> Result<(), crate::Error> {
    Ok(())
}

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "web",
        name: "Web",
        description: "Web virtual module. Used for permission controlling the web dashboard",
        toggleable: false,
        commands_configurable: true,
        virtual_module: true,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![(
            web(),
            indexmap! {
                "use" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("web", "use"),
            },
        )],
        ..Default::default()
    }
}
