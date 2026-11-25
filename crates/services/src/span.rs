use chrono::{NaiveDate, NaiveDateTime, TimeDelta};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub from: NaiveDateTime,
    pub to: NaiveDateTime,
    whole_day: bool,
}

impl Span {
    pub fn today(now: NaiveDateTime) -> Self {
        Self {
            from: midnight(now.date()),
            to: now,
            whole_day: false,
        }
    }

    pub fn day(date: NaiveDate) -> Self {
        let from = midnight(date);
        Self {
            from,
            to: from + TimeDelta::days(1) - TimeDelta::microseconds(1),
            whole_day: true,
        }
    }

    pub fn of(date: NaiveDate, now: NaiveDateTime) -> Self {
        if date == now.date() {
            Self::today(now)
        } else {
            Self::day(date)
        }
    }

    pub fn contains(&self, moment: NaiveDateTime) -> bool {
        self.from <= moment && moment <= self.to
    }

    pub fn label(&self) -> String {
        let end = if self.whole_day {
            "24:00".to_owned()
        } else {
            self.to.format("%H:%M").to_string()
        };
        format!("{} de 00:00 a {end}", self.from.format("%d/%m/%Y"))
    }
}

fn midnight(date: NaiveDate) -> NaiveDateTime {
    date.and_time(chrono::NaiveTime::MIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> NaiveDateTime {
        text.parse().unwrap()
    }

    #[test]
    fn today_ends_at_the_current_time() {
        let now = at("2026-11-25T15:30:00");
        let span = Span::of(now.date(), now);
        assert!(span.contains(at("2026-11-25T00:00:00")));
        assert!(span.contains(now));
        assert!(!span.contains(at("2026-11-25T15:30:01")));
        assert!(!span.contains(at("2026-11-24T23:59:59.999999")));
        assert_eq!(span.label(), "25/11/2026 de 00:00 a 15:30");
    }

    #[test]
    fn a_date_covers_the_whole_day() {
        let now = at("2026-11-25T15:30:00");
        let span = Span::of(at("2026-11-20T00:00:00").date(), now);
        assert!(span.contains(at("2026-11-20T00:00:00")));
        assert!(span.contains(at("2026-11-20T23:59:59.999999")));
        assert!(!span.contains(at("2026-11-21T00:00:00")));
        assert_eq!(span.label(), "20/11/2026 de 00:00 a 24:00");
    }
}
