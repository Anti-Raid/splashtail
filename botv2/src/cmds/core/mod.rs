pub mod help;
pub mod stats;
pub mod ping;

pub fn commands() -> Vec<super::CommandAndPermissions> {
    vec![
        (help::help(), super::CommandExtendedData::default()),
        (help::simplehelp(), super::CommandExtendedData::default()),
        (stats::stats(), super::CommandExtendedData::default()),
        (ping::ping(), super::CommandExtendedData::default()),
    ]
}