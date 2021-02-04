use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: Uuid,
    pub name: String,
    pub completed: bool,
    #[serde(with = "my_date_format")]
    pub created_date: DateTime<Utc>,
    #[serde(with = "my_date_format")]
    pub updated_date: DateTime<Utc>,
}

impl TodoItem {
    pub(crate) fn new(name: &str) -> Self {
        TodoItem {
            id: Uuid::new_v4(),
            name: String::from(name),
            completed: false,
            created_date: Utc::now(),
            updated_date: Utc::now(),
        }
    }

    pub(crate) fn set_completion(&mut self, is_complete: bool) -> &Self {
        self.completed = is_complete;
        self.updated_date = Utc::now();

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_new_item() {
        let name = "test task";
        let item = TodoItem::new(name);

        assert_eq!(item.name, name);
        assert_eq!(item.completed, false);
    }

    #[test]
    fn it_sets_completion_status() {
        let name = "test task";
        let mut item = TodoItem::new(name);

        assert_eq!(item.completed, false);
        item.set_completion(true);
        assert_eq!(item.completed, true);
    }
}

mod my_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
