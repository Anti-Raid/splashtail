mod cmds;
mod autocompletes;

pub fn commands() -> Vec<super::CommandAndPermissions> {
    vec![
        (cmds::limits(), super::CommandExtendedData::default()),
        (cmds::limitactions(), super::CommandExtendedData::default()),
    ]
}