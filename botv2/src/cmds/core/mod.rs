pub mod help;
pub mod stats;
pub mod ping;

pub fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    vec![
        help::help(),
        help::simplehelp(),
        stats::stats(),
        ping::ping(),
    ]
}