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

#[allow(non_snake_case)]
fn acl__modules_modperms() -> poise::Command<crate::Data, crate::Error> {
    /// This command controls if a user can edit a module.
    #[poise::command(prefix_command, slash_command, rename = "acl__modules_modperms")]
    pub async fn base_cmd(_ctx: crate::Context<'_>) -> Result<(), crate::Error> {
        Ok(())
    }

    let mut cmd = base_cmd();
    cmd.name = "acl__modules_modperms".to_string();
    cmd.qualified_name = "acl__modules_modperms".to_string();

    for module in crate::modules::module_ids() {
        let mut subcmd = base_cmd();
        subcmd.description = Some(format!(
            "This command controls if a user can edit the {} module.",
            module
        ));
        subcmd.name = module.to_string();
        subcmd.qualified_name = module.to_string();
        cmd.subcommands.push(subcmd);
    }

    cmd
}

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "access_control",
        name: "Access Control",
        description: "Access Control virtual module. Used for permission controlling the web dashboard and other permission checks",
        toggleable: false,
        commands_toggleable: true,
        virtual_module: true,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                web(),
                indexmap! {
                    "use" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("web", "use"),
                },
            ),
            (
                acl__modules_modperms(),
                {
                    let mut imap = indexmap::IndexMap::new();

                    for module in crate::modules::module_ids() {
                        imap.insert(
                            module,
                            crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "modperm"),
                        );
                    }

                    imap
                }
            ),
        ],
        ..Default::default()
    }
}
