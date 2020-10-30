use std::{env, fs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
// #[serde_json(rename_all = "PascalCase")]
struct Data {
    items: Vec<TodoItem>,
}

#[derive(Debug, Serialize, Deserialize)]
// #[serde_json(rename_all = "PascalCase")]
struct TodoItem {
    name: String,
    completed: char
}

impl TodoItem {
    fn new(name: String) -> TodoItem {
        TodoItem { name, completed: ' ' }
    }
}

struct TodoList {
    list: Vec<TodoItem>
}

impl TodoList {
    fn new() -> TodoList {
        TodoList { list: vec![] }
    }

    fn load_data(&mut self, items: Vec<TodoItem>) {
        for item in items {
            self.list.push(item);
        }
    }

    fn add(&mut self, item: TodoItem) {
        self.list.push(item)
    }

    fn print(&self) {
        for (index, item) in self.list.iter().enumerate() {
            println!("{}. [{}] - {}", index, item.completed, item.name);
        }
    }

    fn toggle_task(&mut self, index: usize) {
        if self.list[index].completed == ' ' {
            self.list[index].completed = 'X';
        } else {
            self.list[index].completed = ' '
        }
    }

    fn remove_task(&mut self, index: usize) {
        self.list.remove(index);
    }
}

enum Command {
    List,
    Add(String),
    TICK(usize),
    REMOVE(usize)
}

fn dump(path_to_file: String, data: Data) {
    let content = serde_json::to_string(&data).expect("Json serialization failed");
    fs::write(path_to_file, content).expect("Data cannot be saved");
}

fn main() {
    let path_to_file = "./src/todos.json";
    let file = fs::read_to_string(path_to_file).expect("Unable to read file");
    let data: Data = serde_json::from_str(file.as_str()).expect("Parsing json has failed");


    let arguments: Vec<String> = env::args().collect();

    let command = match arguments[1].as_str() {
        "list" => Command::List,
        "add" => Command::Add(arguments[2].clone()),
        "tick" => Command::TICK(arguments[2].clone().parse().unwrap()),
        "remove" => Command::REMOVE(arguments[2].clone().parse().unwrap()),
        _ => panic!("Unknown command")
    };

    let mut todo_list = TodoList::new();
    todo_list.load_data(data.items);


    match command {
        Command::List => todo_list.print(),
        Command::Add(task_name) => {
            todo_list.add(TodoItem::new(task_name.to_string()));
            todo_list.print();
            dump(path_to_file.to_string(), Data { items: todo_list.list });
        },
        Command::TICK(task_index) => {
            todo_list.toggle_task(task_index);
            todo_list.print();
            dump(path_to_file.to_string(), Data { items: todo_list.list });
        },
        Command::REMOVE(task_index) => {
            todo_list.remove_task(task_index);
            todo_list.print();
            dump(path_to_file.to_string(), Data { items: todo_list.list });
        }
    }
}
