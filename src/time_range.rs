use std::str::FromStr;
use utils::parse::parse_time_range;
use chrono::NaiveDateTime;

pub struct TimeRange {
    pub from: NaiveDateTime,
    pub to: NaiveDateTime
}

impl FromStr for TimeRange {
    type Err = ();

    fn from_str(data: &str) -> Result<Self, Self::Err> {

        let maybe_time_range = parse_time_range(data);
        match maybe_time_range {
            Some((from, to)) => Ok(TimeRange { from, to }),
            None => Err(())
        }
    }
}
