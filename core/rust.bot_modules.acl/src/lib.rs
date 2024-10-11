use indexmap::indexmap;

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

#[allow(non_snake_case)]
fn acl__modules_modperms(
    module_ids: &[&'static str],
) -> poise::Command<silverpelt::data::Data, silverpelt::Error> {
    #[poise::command(slash_command)]
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

pub struct Module {
    pub module_ids: Vec<&'static str>,
}

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "access_control"
    }

    fn name(&self) -> &'static str {
        "Access Control"
    }

    fn description(&self) -> &'static str {
        "Access Control virtual module. Used for permission controlling the web dashboard, module ACL's and other common ACL's"
    }

    fn toggleable(&self) -> bool {
        false
    }

    fn is_default_enabled(&self) -> bool {
        true // ACL is one of the few modules in std that should be always enabled
    }

    fn virtual_module(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![
            (
                web(),
                indexmap! {
                    "use" => silverpelt::types::CommandExtendedData {
                        virtual_command: true,
                        ..silverpelt::types::CommandExtendedData::kittycat_or_admin("web", "use")
                    },
                },
            ),
            (acl__modules_modperms(&self.module_ids), {
                let mut imap = indexmap::IndexMap::new();

                for module in self.module_ids.iter() {
                    imap.insert(
                        *module,
                        silverpelt::types::CommandExtendedData {
                            virtual_command: true,
                            ..silverpelt::types::CommandExtendedData::none()
                        },
                    );
                }

                imap
            }),
        ]
    }
}
