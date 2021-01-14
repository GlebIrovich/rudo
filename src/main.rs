use std::io;
use std::io::{stdin, stdout};
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::{fs, thread};

use serde::{Deserialize, Serialize};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Terminal;

use crate::utils::StatefulList;

mod utils;

const PATH_TO_FILE: &str = "./src/todos.json";

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    items: Vec<TodoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodoItem {
    name: String,
    completed: bool,
}

#[derive(Copy, Clone)]
enum AppStage {
    Default,
    CreateNewItem,
}

impl TodoItem {
    fn new(name: &str) -> TodoItem {
        TodoItem {
            name: String::from(name),
            completed: false,
        }
    }
}

struct App {
    list: StatefulList<TodoItem>,
    stage: Arc<Mutex<AppStage>>,
    new_item: TodoItem,
}

impl App {
    fn new(items: Vec<TodoItem>) -> App {
        App {
            list: StatefulList::new(items),
            stage: Arc::new(Mutex::new(AppStage::Default)),
            new_item: TodoItem::new(""),
        }
    }

    fn add_new_item(&mut self, item: TodoItem) {
        self.list.items.push(item)
    }

    fn toggle_task(&mut self) {
        match self.list.state.selected() {
            Some(index) => self.list.items[index].completed = !self.list.items[index].completed,
            _ => (),
        }
    }

    fn remove_task(&mut self) {
        match self.list.state.selected() {
            Some(index) => {
                self.list.items.remove(index);
                if self.list.items.len() == 0 {
                    self.list.state.select(Some(0))
                }
            }
            _ => (),
        }
    }

    fn set_stage(&mut self, stage: AppStage) {
        *self.stage.lock().unwrap() = stage;
        self.reset_new_item();
    }

    fn new_item_add_character(&mut self, letter: char) {
        self.new_item.name = format!("{}{}", self.new_item.name, letter);
    }

    fn new_item_remove_character(&mut self) {
        self.new_item.name.pop();
    }

    fn reset_new_item(&mut self) {
        self.new_item = TodoItem::new("")
    }

    fn get_stage_clone(&self) -> AppStage {
        let stage = *self.stage.clone().lock().unwrap();
        stage
    }
}

fn dump(path_to_file: String, data: Data) {
    let content = serde_json::to_string(&data).expect("Json serialization failed");
    fs::write(path_to_file, content).expect("Data cannot be saved");
}

enum TerminalEvent {
    Input(Key),
}

fn main() -> Result<(), io::Error> {
    let stdout = stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Application state
    let mut app = App::new(get_app_data());

    // Clean screen
    terminal.clear().expect("Terminal clean failed");

    let key_events_receiver = spawn_key_event_listener_worker(Arc::clone(&app.stage));

    loop {
        terminal
            .draw(|frame| {
                // Create two chunks with equal horizontal screen space
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(50),
                            Constraint::Percentage(40),
                            Constraint::Percentage(10),
                        ]
                        .as_ref(),
                    )
                    .split(frame.size());

                // Iterate through all elements in the `items` app and append some debug text to it.
                let items: Vec<ListItem> = app
                    .list
                    .items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        let lines = vec![Spans::from(Span::from(format!(
                            "{}. [{}] - {}",
                            index + 1,
                            if item.completed { 'X' } else { ' ' },
                            item.name.clone()
                        )))];
                        ListItem::new(lines)
                            .style(Style::default().fg(Color::Black).bg(Color::White))
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

                let create_block = |title| Block::default().borders(Borders::ALL).title(title);

                // Add input block
                match *app.stage.lock().unwrap() {
                    AppStage::CreateNewItem => {
                        let input_block = Paragraph::new(format!("{}", app.new_item.name))
                            .block(create_block("New todo item"))
                            .alignment(Alignment::Left);

                        frame.render_widget(input_block, chunks[1]);
                    }
                    _ => (),
                }

                // Debug application state

                let paragraph = Paragraph::new(format!(
                    "{}",
                    match *app.stage.lock().unwrap() {
                        AppStage::CreateNewItem => format!("new item: {}", app.new_item.name),
                        AppStage::Default => "default".to_string(),
                    }
                ))
                .block(create_block("App stage"))
                .alignment(Alignment::Left);

                frame.render_widget(paragraph, chunks[2]);
            })
            .expect("Terminal draw failed");

        match key_events_receiver.recv().unwrap() {
            TerminalEvent::Input(Key::Char('q')) => match app.get_stage_clone() {
                AppStage::CreateNewItem => app.new_item_add_character('q'),
                _ => {
                    terminal.clear().expect("Terminal clean failed");
                    dump(
                        PATH_TO_FILE.to_string(),
                        Data {
                            items: app.list.items,
                        },
                    );
                    break Result::Ok(());
                }
            },
            TerminalEvent::Input(Key::Char('d')) => match app.get_stage_clone() {
                AppStage::CreateNewItem => app.new_item_add_character('d'),
                _ => app.remove_task(),
            },
            TerminalEvent::Input(Key::Backspace) => match app.get_stage_clone() {
                AppStage::CreateNewItem => app.new_item_remove_character(),
                _ => (),
            },
            TerminalEvent::Input(Key::Down) => app.list.next(),
            TerminalEvent::Input(Key::Up) => app.list.previous(),
            TerminalEvent::Input(Key::Char('\n')) => match app.get_stage_clone() {
                AppStage::CreateNewItem => {
                    app.add_new_item(app.new_item.clone());
                    app.reset_new_item();
                    app.set_stage(AppStage::Default);
                }
                _ => app.toggle_task(),
            },
            TerminalEvent::Input(Key::Char(' ')) => match app.get_stage_clone() {
                AppStage::CreateNewItem => app.new_item_add_character(' '),
                _ => app.toggle_task(),
            },

            TerminalEvent::Input(Key::Char('n')) => match app.get_stage_clone() {
                AppStage::CreateNewItem => app.new_item_add_character('n'),
                _ => app.set_stage(AppStage::CreateNewItem),
            },

            TerminalEvent::Input(Key::Esc) => app.set_stage(AppStage::Default),
            TerminalEvent::Input(Key::Char(letter)) => app.new_item_add_character(letter),
            _ => (),
        };
    }
}

fn spawn_key_event_listener_worker(app_stage: Arc<Mutex<AppStage>>) -> Receiver<TerminalEvent> {
    let stdin = stdin();

    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        //detecting keydown events
        for event in stdin.keys() {
            match event.unwrap() {
                Key::Char('q') => match *app_stage.lock().unwrap() {
                    AppStage::CreateNewItem => {
                        sender.send(TerminalEvent::Input(Key::Char('q'))).unwrap()
                    }
                    _ => break,
                },
                key => sender.send(TerminalEvent::Input(key)).unwrap(),
            }
        }
    });

    receiver
}

fn get_app_data() -> Vec<TodoItem> {
    let file = fs::read_to_string(PATH_TO_FILE).expect("Unable to read file");
    let data: Data = serde_json::from_str(file.as_str()).expect("Parsing json has failed");

    data.items
}
