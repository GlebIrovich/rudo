mod utils;

use crate::utils::{
    StatefulList,
};

use std::{fs, thread};
use serde::{Deserialize, Serialize};
use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::{TermionBackend};
use tui::widgets::{Block, Borders, ListItem, List, Paragraph};
use tui::layout::{Layout, Constraint, Direction, Alignment};
use termion::event::Key;
use termion::input::TermRead;
use std::io::{stdin, stdout};
use std::sync::mpsc;
use tui::text::{Span, Spans};
use tui::style::{Style, Modifier, Color};

const PATH_TO_FILE: &str = "./src/todos.json";

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    items: Vec<TodoItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoItem {
    name: String,
    completed: char
}

enum AppStage {
    Default,
    CreateNewItem
}

impl TodoItem {
    fn new(name: String) -> TodoItem {
        TodoItem { name, completed: ' ' }
    }
}

struct App {
    list: StatefulList<TodoItem>,
    stage: AppStage
}

impl App {
    fn new(items: Vec<TodoItem>) -> App {
        App {
            list: StatefulList::new(items),
            stage: AppStage::Default
        }
    }

    fn add(&mut self, item: TodoItem) {
        self.list.items.push(item)
    }

    fn toggle_task(&mut self) {
        match self.list.state.selected() {
            Some(index) => {
                if self.list.items[index].completed == ' ' {
                    self.list.items[index].completed = 'X';
                } else {
                    self.list.items[index].completed = ' ';
                }
            },
            _ => ()
        }
    }

    fn remove_task(&mut self) {
        match self.list.state.selected() {
            Some(index) => {
                self.list.items.remove(index);
            }
            _ => ()
        }
    }

    fn set_stage(&mut self, stage: AppStage) {
        self.stage = stage;
    }
}

fn dump(path_to_file: String, data: Data) {
    let content = serde_json::to_string(&data).expect("Json serialization failed");
    fs::write(path_to_file, content).expect("Data cannot be saved");
}

enum TerminalEvent {
    Quit,
    Next,
    Previous,
    Delete,
    Tick,
    StageChange(AppStage)
}

fn main() -> Result<(), io::Error> {
    let stdin = stdin();
    let stdout = stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Application state
    let mut app = get_app_data();

    // Clean screen
    terminal.clear();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        //detecting keydown events
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('h') => println!("Hello world!"),
                Key::Char('q') => {
                    tx.send(TerminalEvent::Quit).unwrap();
                    break;
                },
                Key::Char('d') => tx.send(TerminalEvent::Delete).unwrap(),
                Key::Down => tx.send(TerminalEvent::Next).unwrap(),
                Key::Up => tx.send(TerminalEvent::Previous).unwrap(),
                Key::Char('\n') => tx.send(TerminalEvent::Tick).unwrap(),
                Key::Char(' ') => tx.send(TerminalEvent::Tick).unwrap(),

                Key::Char('n') => tx.send(TerminalEvent::StageChange(AppStage::CreateNewItem)).unwrap(),
                Key::Esc => tx.send(TerminalEvent::StageChange(AppStage::Default)).unwrap(),
                key => println!("{:?}", key),
            }
        }
    });

    loop {
        terminal.draw(|frame| {
            // Create two chunks with equal horizontal screen space
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(40),
                    Constraint::Percentage(10)
                ].as_ref())
                .split(frame.size());

            // Iterate through all elements in the `items` app and append some debug text to it.
            let items: Vec<ListItem> = app
                .list
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let lines = vec![Spans::from(
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
            frame.render_stateful_widget(items, chunks[0], &mut app.list.state);

            let create_block = |title| {
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
            };

            // Add input block
            match app.stage {
                AppStage::CreateNewItem => {
                    let input_block = Paragraph::new("")
                        .block(create_block("New todo item"))
                        .alignment(Alignment::Left);

                    frame.render_widget(input_block, chunks[1]);
                },
                _ => ()
            }

            // Debug application state

            let paragraph = Paragraph::new(
                format!("{}", match app.stage {
                    AppStage::CreateNewItem => "new item",
                    AppStage::Default => "default"
                })
            )
                .block(create_block("App stage"))
                .alignment(Alignment::Left);

            frame.render_widget(paragraph, chunks[2]);
        });

        match rx.recv().unwrap() {
            TerminalEvent::Quit => {
                terminal.clear();
                dump(PATH_TO_FILE.to_string(), Data {items: app.list.items});
                break Result::Ok(())
            },
            TerminalEvent::Next => app.list.next(),
            TerminalEvent::Previous => app.list.previous(),
            TerminalEvent::Delete => app.remove_task(),
            TerminalEvent::Tick => app.toggle_task(),
            TerminalEvent::StageChange(stage) => app.set_stage(stage),
        }
    }
}

fn get_app_data() -> App {
    let file = fs::read_to_string(PATH_TO_FILE).expect("Unable to read file");
    let data: Data = serde_json::from_str(file.as_str()).expect("Parsing json has failed");

    let mut app = App::new(data.items);

    app
}