use crate::todo_item::TodoItem;
use crate::utils::StatefulList;
use std::fmt::{Display, Formatter, Result};
use std::sync::{Arc, Mutex};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AppStage {
    Default,
    CreateNewItem,
    Filter,
}

pub struct App {
    pub list: StatefulList<TodoItem>,
    pub items: Vec<TodoItem>,
    pub stage: Arc<Mutex<AppStage>>,
    pub new_item_name: String,
    pub filter_term: String,
    pub sorting_order: SortingOrder,
}

impl App {
    pub fn new(items: Vec<TodoItem>) -> App {
        let mut app = App {
            items: items.iter().cloned().collect(),
            list: StatefulList::new(items),
            stage: Arc::new(Mutex::new(AppStage::Default)),
            new_item_name: String::new(),
            sorting_order: SortingOrder::Ascending,
            filter_term: String::new(),
        };

        app.sort_by_date();
        app.select_first_task_or_none();
        app
    }

    pub fn add_new_item(&mut self) {
        self.items.push(TodoItem::new(&self.new_item_name));
        self.list = StatefulList::new(self.items.iter().cloned().collect());
    }

    pub fn toggle_task(&mut self) {
        match self.get_selected_item_index() {
            None => {}
            Some(index) => {
                let new_status = !self.items[index].completed;
                let todo_item = &mut self.items[index];
                todo_item.set_completion(new_status);

                self.apply_filter();
            }
        }
    }

    pub fn remove_task(&mut self) {
        match self.get_selected_item_index() {
            None => {}
            Some(index) => {
                self.items.remove(index);
                self.list = StatefulList::new(self.items.iter().cloned().collect());
                self.select_first_task_or_none();
            }
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

    pub fn filter_term_add_character(&mut self, letter: char) {
        self.filter_term = format!("{}{}", self.filter_term, letter);
        self.apply_filter();
    }

    pub fn filter_term_remove_character(&mut self) {
        self.filter_term.pop();
        self.apply_filter();
    }

    pub fn get_stage_clone(&self) -> AppStage {
        let stage = *self.stage.clone().lock().unwrap();
        stage
    }

    pub fn apply_filter(&mut self) {
        let items = self
            .items
            .iter()
            .filter(|item| {
                item.name
                    .to_lowercase()
                    .contains(&self.filter_term.to_lowercase())
            })
            .cloned()
            .collect();

        self.list = StatefulList::new(items);
        self.select_first_task_or_none();
    }

    pub fn get_filtered_items(&self) -> Vec<&TodoItem> {
        self.list
            .items
            .iter()
            .filter(|item| {
                item.name
                    .to_lowercase()
                    .contains(&self.filter_term.to_lowercase())
            })
            .collect()
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

    fn get_selected_item_index(&self) -> Option<usize> {
        match self.list.get_selected_item() {
            Some(selected_item) => self
                .items
                .iter()
                .position(|item| item.id == selected_item.id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TASK_A_NAME: &str = "task A";
    static TASK_B_NAME: &str = "task B";

    #[test]
    fn it_creates_app() {
        let items = create_todo_items();
        let app = App::new(items.clone());

        assert_eq!(app.items.len(), items.len());
        assert_eq!(app.items[0].id, items[0].id);
        assert_eq!(app.sorting_order, SortingOrder::Ascending);
        assert_eq!(*app.stage.lock().unwrap(), AppStage::Default);
        assert_eq!(app.new_item_name, "");
        assert_eq!(app.filter_term, "");

        // Correct item is selected
        assert_eq!(app.list.get_selected_item().unwrap().id, items[0].id);
    }

    fn create_todo_items() -> Vec<TodoItem> {
        vec![
            TodoItem::new(TASK_A_NAME.clone()),
            TodoItem::new(TASK_B_NAME.clone()),
        ]
    }
}

#[derive(PartialEq, Debug)]
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
