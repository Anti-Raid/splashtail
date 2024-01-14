mod cmds;
mod autocompletes;

use indexmap::indexmap;

pub fn commands() -> Vec<super::CommandAndPermissions> {
    vec![
        (cmds::limits(), indexmap! {
            "add" => super::CommandExtendedData::kittycat_simple("limits", "add"),
            "view" => super::CommandExtendedData::kittycat_simple("limits", "view"),
            "remove" => super::CommandExtendedData::kittycat_simple("limits", "remove"),
            "hit" => super::CommandExtendedData::kittycat_simple("limits", "hit"),
        }),
        (cmds::limitactions(), super::CommandExtendedData::none()),
    ]
}