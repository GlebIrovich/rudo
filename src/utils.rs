use tui::widgets::ListState;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T>
where
    T: Clone,
{
    pub fn new(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let number_of_items = self.items.len();
        let item = match self.state.selected() {
            Some(i) => {
                if i >= number_of_items - 1 {
                    Some(0)
                } else {
                    Some(i + 1)
                }
            }
            None => {
                if number_of_items > 0 {
                    Some(0)
                } else {
                    None
                }
            }
        };
        self.state.select(item);
    }

    pub fn previous(&mut self) {
        let number_of_items = self.items.len();
        let item = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    Some(number_of_items - 1)
                } else {
                    Some(i - 1)
                }
            }
            None => {
                if number_of_items > 0 {
                    Some(number_of_items - 1)
                } else {
                    None
                }
            }
        };
        self.state.select(item);
    }

    pub fn get_selected_item(&self) -> Option<T> {
        match self.state.selected() {
            Some(index) => Some(self.items[index].clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gets_selected_item() {
        let mut list = StatefulList::new(vec!["a", "b", "c"]);

        assert_eq!(list.state.selected(), None);
        list.next();
        assert_eq!(list.get_selected_item(), Some("a"));
        list.next();
        assert_eq!(list.get_selected_item(), Some("b"));
        list.previous();
        assert_eq!(list.get_selected_item(), Some("a"));
    }
}
