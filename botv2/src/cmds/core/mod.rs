pub mod help;
pub mod stats;
pub mod ping;

pub fn module() -> super::Module {
    super::Module {
        id: "core",
        name: "Core Commands",
        commands: vec![
            (help::help(), super::CommandExtendedData::none()),
            (help::simplehelp(), super::CommandExtendedData::none()),
            (stats::stats(), super::CommandExtendedData::none()),
            (ping::ping(), super::CommandExtendedData::none()),
        ],
    }
}