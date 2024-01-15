mod cmds;
use indexmap::indexmap;

pub fn commands() -> Vec<super::CommandAndPermissions> {
    vec![
        (cmds::backups(), indexmap! {
            "" => super::CommandExtendedData::kittycat_simple("backups", "*"),
            "create" => super::CommandExtendedData::kittycat_simple("backups", "create"),
        }),
    ]
}
