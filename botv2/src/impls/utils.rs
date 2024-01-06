use sqlx::postgres::types::PgInterval;

pub fn parse_pg_interval(i: PgInterval) -> String {
    let seconds = i.microseconds / 1000000 + ((i.days * 86400) as i64) + ((i.months * 2628000) as i64);

    let dur = std::time::Duration::from_secs(seconds.try_into().unwrap_or_default());

    format!("{:?}", dur)
}

#[derive(poise::ChoiceParameter)]
pub enum Unit {
    #[name = "Seconds"]
    Seconds,
    #[name = "Minutes"]
    Minutes,
    #[name = "Hours"]
    Hours,
    #[name = "Days"]
    Days,
}

impl Unit {
    pub fn to_seconds(&self) -> i64 {
        match self {
            Unit::Seconds => 1,
            Unit::Minutes => 60,
            Unit::Hours => 3600,
            Unit::Days => 86400,
        }
    }
}
