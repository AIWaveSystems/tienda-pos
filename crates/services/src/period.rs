use chrono::{Datelike, NaiveDate, TimeDelta};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Period {
    Today,
    Week,
    #[default]
    Fortnight,
    Month,
    ThisMonth,
    Year,
}

impl Period {
    pub const ALL: [Self; 6] = [
        Self::Today,
        Self::Week,
        Self::Fortnight,
        Self::Month,
        Self::ThisMonth,
        Self::Year,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Today => "Hoy",
            Self::Week => "7 dias",
            Self::Fortnight => "14 dias",
            Self::Month => "30 dias",
            Self::ThisMonth => "Este mes",
            Self::Year => "365 dias",
        }
    }

    pub fn days(self, today: NaiveDate) -> i64 {
        match self {
            Self::Today => 1,
            Self::Week => 7,
            Self::Fortnight => 14,
            Self::Month => 30,
            Self::ThisMonth => i64::from(today.day()),
            Self::Year => 365,
        }
    }

    pub fn start(self, today: NaiveDate) -> NaiveDate {
        today - TimeDelta::days(self.days(today) - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periods_include_today() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap();
        assert_eq!(Period::Today.start(today), today);
        assert_eq!(Period::ThisMonth.start(today).day(), 1);
        assert_eq!(Period::Week.start(today).day(), 19);
    }
}
