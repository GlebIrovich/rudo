use std::{env, fs, thread};
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
        if self.list.len() > 0 {
            for (index, item) in self.list.iter().enumerate() {
                println!("{}. [{}] - {}", index, item.completed, item.name);
            }
        } else {
            println!("There is no todos")
        }
    }

    fn toggle_task(&mut self, index: usize) -> bool {
        return if self.list[index].completed == ' ' {
            self.list[index].completed = 'X';
            true
        } else {
            self.list[index].completed = ' ';
            false
        }
    }

    fn remove_task(&mut self, index: usize) {
        self.list.remove(index);
    }

    fn get_task(&mut self, index: usize) -> &TodoItem {
        &self.list[index]
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

use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::{TermionBackend, Backend};
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};
use termion::event::Key;
use termion::input::TermRead;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc;

enum Event {
    QUIT
}

fn main() -> Result<(), io::Error> {
    let stdin = stdin();
    let stdout = stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clean screen
    terminal.clear();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        //detecting keydown events
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('h') => println!("Hello world!"),
                Key::Char('q') => {
                    tx.send(Event::QUIT).unwrap();
                    break;
                },
                _ => (),
            }

            // stdout.flush().unwrap();
        }
    });

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(10), Constraint::Min(0)].as_ref())
                .split(f.size());
            let block = Block::default()
                .title("Block")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[1]);
        });

        match rx.recv().unwrap() {
            Event::QUIT => {
                break
                Result::Ok(())
            },
        }
    }
}


fn main1() {
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
            println!("{} added!", task_name.to_string());

            dump(path_to_file.to_string(), Data { items: todo_list.list });
        },
        Command::TICK(task_index) => {
            let done = todo_list.toggle_task(task_index);
            let task = todo_list.get_task(task_index);
            if done {
                println!("{} is set to DONE.", task.name);
            } else {
                println!("{} is set to NOT DONE.", task.name);
            }

            dump(path_to_file.to_string(), Data { items: todo_list.list });
        },
        Command::REMOVE(task_index) => {
            todo_list.remove_task(task_index);
            println!("Task is removed");

            dump(path_to_file.to_string(), Data { items: todo_list.list });
        }
    }
}
