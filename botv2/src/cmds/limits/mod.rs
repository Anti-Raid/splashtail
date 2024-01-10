mod cmds;
mod autocompletes;

pub fn commands() -> Vec<super::CommandAndPermissions> {
    vec![
        (cmds::limits(), super::CommandExtendedData::none()),
        (cmds::limitactions(), super::CommandExtendedData::none()),
    ]
}