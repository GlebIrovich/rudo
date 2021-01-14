use crate::utils::StatefulList;
use serde::{Deserialize, Serialize};
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
}

impl TodoItem {
    fn new(name: &str) -> TodoItem {
        TodoItem {
            name: String::from(name),
            completed: false,
        }
    }
}

pub struct App {
    pub list: StatefulList<TodoItem>,
    pub stage: Arc<Mutex<AppStage>>,
    pub new_item: TodoItem,
}

impl App {
    pub fn new(items: Vec<TodoItem>) -> App {
        let mut app = App {
            list: StatefulList::new(items),
            stage: Arc::new(Mutex::new(AppStage::Default)),
            new_item: TodoItem::new(""),
        };

        app.list.next();
        app
    }

    pub fn add_new_item(&mut self, item: TodoItem) {
        self.list.items.push(item)
    }

    pub fn toggle_task(&mut self) {
        match self.list.state.selected() {
            Some(index) => self.list.items[index].completed = !self.list.items[index].completed,
            _ => (),
        }
    }

    pub fn remove_task(&mut self) {
        match self.list.state.selected() {
            Some(index) => {
                self.list.items.remove(index);
                if self.list.items.len() == 0 {
                    self.list.state.select(None)
                }
            }
            _ => (),
        }
    }

    pub fn set_stage(&mut self, stage: AppStage) {
        *self.stage.lock().unwrap() = stage;
        self.reset_new_item();
    }

    pub fn new_item_add_character(&mut self, letter: char) {
        self.new_item.name = format!("{}{}", self.new_item.name, letter);
    }

    pub fn new_item_remove_character(&mut self) {
        self.new_item.name.pop();
    }

    pub fn reset_new_item(&mut self) {
        self.new_item = TodoItem::new("")
    }

    pub fn get_stage_clone(&self) -> AppStage {
        let stage = *self.stage.clone().lock().unwrap();
        stage
    }
}
