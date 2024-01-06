mod cmds;
mod autocompletes;

pub fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    vec![
        cmds::limits(),
        cmds::limitactions(),
    ]
}