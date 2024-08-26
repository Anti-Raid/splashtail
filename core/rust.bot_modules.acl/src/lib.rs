use indexmap::indexmap;

/// Web interface access control
#[poise::command(prefix_command, slash_command, subcommands("web_use"))]
pub async fn web(_ctx: silverpelt::Context<'_>) -> Result<(), silverpelt::Error> {
    Ok(())
}

/// This command controls if a user can use the web interface.
#[poise::command(prefix_command, slash_command, rename = "use")]
pub async fn web_use(_ctx: silverpelt::Context<'_>) -> Result<(), silverpelt::Error> {
    Ok(())
}

#[allow(non_snake_case)]
fn acl__modules_modperms(
    module_ids: &[&'static str],
) -> poise::Command<silverpelt::data::Data, silverpelt::Error> {
    #[poise::command(prefix_command, slash_command)]
    pub async fn base_cmd(_ctx: silverpelt::Context<'_>) -> Result<(), silverpelt::Error> {
        Ok(())
    }

    let mut cmd = base_cmd();
    cmd.name = "acl__modules_modperms".to_string();
    cmd.qualified_name = "acl__modules_modperms".to_string();
    cmd.description =
        Some("This command controls if a user can edit module permissions.".to_string());

    for module in module_ids {
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

pub fn module(module_ids: Vec<&'static str>) -> silverpelt::Module {
    // Add ACL to the list of modules
    let mut module_ids = module_ids;
    module_ids.push("acl");

    silverpelt::Module {
        id: "access_control",
        name: "Access Control",
        description: "Access Control virtual module. Used for permission controlling the web dashboard and other common ACL's",
        toggleable: false,
        commands_toggleable: true,
        virtual_module: true,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                web(),
                indexmap! {
                    "use" => silverpelt::types::CommandExtendedData::kittycat_or_admin("web", "use"),
                },
            ),
            (
                acl__modules_modperms(&module_ids),
                {
                    let mut imap = indexmap::IndexMap::new();

                    for module in module_ids {
                        imap.insert(
                            module,
                            silverpelt::types::CommandExtendedData::none(),
                        );
                    }

                    imap
                }
            ),
        ],
        ..Default::default()
    }
}
