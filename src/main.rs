mod utils;

use crate::utils::{
    StatefulList,
};

use std::{env, fs, thread};
use serde::{Deserialize, Serialize};
use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::{TermionBackend, Backend};
use tui::widgets::{Widget, Block, Borders, ListItem, List};
use tui::layout::{Layout, Constraint, Direction};
use termion::event::Key;
use termion::input::TermRead;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use tui::text::{Span, Spans};
use tui::style::{Style, Modifier, Color};

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

struct App {
    items: StatefulList<TodoItem>,
}

impl App {
    fn new(items: Vec<TodoItem>) -> App {
        App {
            items: StatefulList::with_items(items),
        }
    }
}

struct TodoList {
    list: Vec<TodoItem>
}

impl TodoList {
    fn new() -> TodoList {
        TodoList { list: vec![] }
    }

    fn with_data(&mut self, items: Vec<TodoItem>) {
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

enum AppEvent {
    QUIT,
    NEXT,
    PREVIOUS,
}

fn main() -> Result<(), io::Error> {
    let stdin = stdin();
    let stdout = stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Application state
    let mut todo_list = get_todo_list();
    let mut app = App::new(todo_list.list);

    // Clean screen
    terminal.clear();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        //detecting keydown events
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('h') => println!("Hello world!"),
                Key::Char('q') => {
                    tx.send(AppEvent::QUIT).unwrap();
                    break;
                },
                Key::Down => tx.send(AppEvent::NEXT).unwrap(),
                Key::Up => tx.send(AppEvent::PREVIOUS).unwrap(),
                _ => (),
            }
        }
    });

    loop {
        terminal.draw(|frame| {
            // Create two chunks with equal horizontal screen space
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(frame.size());

            // Iterate through all elements in the `items` app and append some debug text to it.
            let items: Vec<ListItem> = app
                .items
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let mut lines = vec![Spans::from(
                        Span::from(
                            format!("{}. [{}] - {}", index + 1, item.completed, item.name.clone())
                        )
                    )];
                    ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
                })
                .collect();

            // Create a List from all list items and highlight the currently selected one
            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Todo"))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            // We can now render the item list
            frame.render_stateful_widget(items, chunks[0], &mut app.items.state);
        });

        match rx.recv().unwrap() {
            AppEvent::QUIT => {
                terminal.clear();
                break
                Result::Ok(())
            },
            AppEvent::NEXT => app.items.next(),
            AppEvent::PREVIOUS => app.items.previous(),
        }
    }
}

fn get_todo_list() -> TodoList {
    let path_to_file = "./src/todos.json";
    let file = fs::read_to_string(path_to_file).expect("Unable to read file");
    let data: Data = serde_json::from_str(file.as_str()).expect("Parsing json has failed");

    let mut todo_list = TodoList::new();
    todo_list.with_data(data.items);

    todo_list
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
    todo_list.with_data(data.items);


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
