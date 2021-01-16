use crate::utils::StatefulList;
use chrono::{DateTime, Utc};
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Result};
use std::sync::{Arc, Mutex};

#[derive(Copy, Clone)]
pub enum AppStage {
    Default,
    CreateNewItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub name: String,
    pub completed: bool,
    #[serde(with = "my_date_format")]
    pub created_date: DateTime<Utc>,
    #[serde(with = "my_date_format")]
    pub updated_date: DateTime<Utc>,
}

impl TodoItem {
    fn new(name: &str) -> Self {
        TodoItem {
            name: String::from(name),
            completed: false,
            created_date: Utc::now(),
            updated_date: Utc::now(),
        }
    }

    fn update_name(&mut self, name: &str) -> &Self {
        self.name = String::from(name);
        self.updated_date = Utc::now();

        self
    }

    fn set_completion(&mut self, is_complete: bool) -> &Self {
        self.completed = is_complete;
        self.updated_date = Utc::now();

        self
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

pub struct App {
    pub list: StatefulList<TodoItem>,
    pub stage: Arc<Mutex<AppStage>>,
    pub new_item_name: String,
    pub sorting_order: SortingOrder,
}

impl App {
    pub fn new(items: Vec<TodoItem>) -> App {
        let mut app = App {
            list: StatefulList::new(items),
            stage: Arc::new(Mutex::new(AppStage::Default)),
            new_item_name: String::new(),
            sorting_order: SortingOrder::Ascending,
        };

        app.sort_by_date();
        app.select_first_task_or_none();
        app
    }

    pub fn add_new_item(&mut self) {
        self.list.items.push(TodoItem::new(&self.new_item_name));
        self.sort_by_date();
    }

    pub fn toggle_task(&mut self) {
        match self.list.state.selected() {
            Some(index) => {
                let new_status = !self.list.items[index].completed;
                let todo_item = &mut self.list.items[index];
                todo_item.set_completion(new_status);
            }
            _ => (),
        };
    }

    pub fn remove_task(&mut self) {
        match self.list.state.selected() {
            Some(index) => {
                self.list.items.remove(index);
                self.select_first_task_or_none();
            }
            _ => (),
        }
    }

    pub fn set_stage(&mut self, stage: AppStage) {
        *self.stage.lock().unwrap() = stage;
        self.reset_new_item_name();
    }

    fn set_sorting_order(&mut self, order: SortingOrder) {
        self.sorting_order = order;
        self.sort_by_date();
    }

    pub fn toggle_sorting(&mut self) {
        match self.sorting_order {
            SortingOrder::Ascending => self.set_sorting_order(SortingOrder::Descending),
            SortingOrder::Descending => self.set_sorting_order(SortingOrder::Ascending),
        }
    }

    pub fn new_item_add_character(&mut self, letter: char) {
        self.new_item_name = format!("{}{}", self.new_item_name, letter);
    }

    pub fn new_item_remove_character(&mut self) {
        self.new_item_name.pop();
    }

    pub fn reset_new_item_name(&mut self) {
        self.new_item_name = String::new()
    }

    pub fn get_stage_clone(&self) -> AppStage {
        let stage = *self.stage.clone().lock().unwrap();
        stage
    }

    fn sort_by_date(&mut self) {
        let order = &self.sorting_order;
        self.list.items.sort_by(|item_a, item_b| match order {
            SortingOrder::Ascending => item_a.updated_date.cmp(&item_b.updated_date),
            SortingOrder::Descending => item_a.updated_date.cmp(&item_b.updated_date).reverse(),
        });
    }

    fn select_first_task_or_none(&mut self) {
        if self.list.items.len() > 0 {
            self.list.state.select(Some(0));
        } else {
            self.list.state.select(None);
        }
    }
}

pub enum SortingOrder {
    Ascending,
    Descending,
}

impl Display for SortingOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match &self {
                SortingOrder::Ascending => "ascending",
                SortingOrder::Descending => "descending",
            }
        )
    }
}
