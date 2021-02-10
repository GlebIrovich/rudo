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
    pub stage: Arc<Mutex<AppStage>>,
    pub new_item_name: String,
    pub filter_term: String,
    pub sorting_order: SortingOrder,
}

impl App {
    pub fn new(items: Vec<TodoItem>) -> App {
        let mut app = App {
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
        if self.new_item_name.len() == 0 {
            return;
        }
        self.list.items.push(TodoItem::new(&self.new_item_name));
    }

    pub fn toggle_task(&mut self) {
        match self.list.get_selected_item() {
            Some(selected_item) => {
                for item in &mut self.list.items {
                    if item.id == selected_item.id {
                        item.set_completion(!item.completed);
                    }
                }
            }
            _ => {}
        };
    }

    pub fn remove_task(&mut self) {
        match self.list.get_selected_item() {
            Some(selected_item) => {
                let filtered_items: Vec<TodoItem> = self
                    .list
                    .items
                    .iter()
                    .filter(|item| item.id != selected_item.id)
                    .cloned()
                    .collect();
                self.list = StatefulList::new(filtered_items);
                self.select_first_task_or_none();
            }
            _ => {}
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
    }

    pub fn filter_term_remove_character(&mut self) {
        self.filter_term.pop();
    }

    pub fn get_stage_clone(&self) -> AppStage {
        let stage = *self.stage.clone().lock().unwrap();
        stage
    }

    pub fn get_filtered_items(&self) -> Vec<(usize, TodoItem)> {
        let mut items: Vec<(usize, TodoItem)> = vec![];
        for (index, item) in self.list.items.iter().enumerate() {
            if item
                .name
                .to_lowercase()
                .contains(&self.filter_term.to_lowercase())
            {
                items.push((index, item.clone()));
            }
        }

        items
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

#[cfg(test)]
mod tests {
    use super::*;

    static TASK_A_NAME: &str = "A";
    static TASK_B_NAME: &str = "B";

    #[test]
    fn it_creates_app() {
        let items = create_todo_items();
        let app = App::new(items.clone());

        assert_eq!(app.list.items.len(), items.len());
        assert_eq!(app.list.items[0].id, items[0].id);
        assert_eq!(app.sorting_order, SortingOrder::Ascending);
        assert_eq!(*app.stage.lock().unwrap(), AppStage::Default);
        assert_eq!(app.new_item_name, "");
        assert_eq!(app.filter_term, "");

        // Correct item is selected
        assert_eq!(app.list.get_selected_item().unwrap().id, items[0].id);
    }

    #[test]
    fn it_add_new_item() {
        let mut app = App::new(vec![]);
        app.new_item_add_character('a');

        assert_eq!(app.new_item_name, "a");

        app.add_new_item();
        assert_eq!(app.list.items[0].name, "a");
        assert_eq!(app.list.items[0].name, "a");
    }

    #[test]
    fn it_should_not_add_empty_item() {
        let mut app = App::new(vec![]);

        app.add_new_item();
        assert_eq!(app.list.items.len(), 0);
    }

    #[test]
    fn it_removes_selected_item() {
        let items = create_todo_items();
        let mut app = App::new(vec![items[0].clone()]);

        app.remove_task();
        assert_eq!(app.list.items.len(), 0);
    }

    #[test]
    fn it_toggles_sorting() {
        let items = create_todo_items();
        let mut app = App::new(vec![items[0].clone()]);

        app.toggle_sorting();
        assert_eq!(app.sorting_order, SortingOrder::Descending);
    }

    #[test]
    fn it_filters_items() {
        let items = create_todo_items();
        let mut app = App::new(items.clone());

        app.filter_term_add_character('a');
        assert_eq!(app.filter_term, "a");
        assert_eq!(app.list.items.len(), 2);
        assert_eq!(app.get_filtered_items().len(), 1);
    }

    #[test]
    fn it_should_keep_initial_list_enumeration() {
        let items = create_todo_items();
        let mut app = App::new(items.clone());

        app.filter_term_add_character('b');
        assert_eq!(app.filter_term, "b");
        assert_eq!(app.get_filtered_items()[0].0, 1);
        assert_eq!(app.get_filtered_items()[0].1.name, TASK_B_NAME);
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
