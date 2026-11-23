use chrono::{DurationRound, Local, NaiveDateTime, TimeDelta};

pub fn now() -> NaiveDateTime {
    let current = Local::now().naive_local();
    current
        .duration_trunc(TimeDelta::microseconds(1))
        .unwrap_or(current)
}

pub mod optional {
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f";

    pub fn serialize<S: Serializer>(
        value: &Option<NaiveDateTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(moment) => serializer.serialize_str(&moment.format(FORMAT).to_string()),
            None => serializer.serialize_str(""),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<NaiveDateTime>, D::Error> {
        let text = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
        Ok(text.parse().ok())
    }
}
