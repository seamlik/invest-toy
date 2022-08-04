use chrono::prelude::*;
use serde::Serializer;

pub fn serialize<S>(src: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match src {
        Some(time) => serializer.serialize_some(time),
        None => serializer.serialize_none(),
    }
}
