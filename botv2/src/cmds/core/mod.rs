pub mod help;
pub mod stats;
pub mod ping;

pub fn commands() -> Vec<super::CommandAndPermissions> {
    vec![
        (help::help(), super::CommandExtendedData::none()),
        (help::simplehelp(), super::CommandExtendedData::none()),
        (stats::stats(), super::CommandExtendedData::none()),
        (ping::ping(), super::CommandExtendedData::none()),
    ]
}