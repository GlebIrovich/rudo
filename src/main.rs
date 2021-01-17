use std::io;
use std::io::{stdin, stdout, Stdout};
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::{fs, thread};

use serde::{Deserialize, Serialize};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Terminal;

use crate::app::{App, AppStage, TodoItem};

mod app;
mod utils;

const PATH_TO_FILE: &str = "./src/todos.json";

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    items: Vec<TodoItem>,
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
        let constraint: Vec<Constraint> = match *app.stage.lock().unwrap() {
            AppStage::Default | AppStage::Filter => vec![Constraint::Percentage(100)],
            AppStage::CreateNewItem => vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        };

        terminal
            .draw(|frame| {
                // Create two chunks with equal horizontal screen space
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(constraint.as_ref())
                    .split(frame.size());

                // Iterate through all elements in the `items` app and append some debug text to it.
                let items: Vec<ListItem> = app
                    .list
                    .items
                    .iter()
                    .filter(|item| {
                        item.name
                            .to_lowercase()
                            .contains(&app.filter_term.to_lowercase())
                    })
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
                let title = format!(
                    "Todo list - Sorting: {} - Filter: {}",
                    app.sorting_order,
                    if app.filter_term == "" {
                        "None"
                    } else {
                        &app.filter_term
                    }
                );

                let border_color = match *app.stage.lock().unwrap() {
                    AppStage::Default | AppStage::Filter => Color::Green,
                    AppStage::CreateNewItem => Color::Reset,
                };

                let items = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(border_color))
                            .title(title),
                    )
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
                match app.get_stage_clone() {
                    AppStage::CreateNewItem => {
                        let input_block = Paragraph::new(format!("{}", app.new_item_name))
                            .block(
                                create_block("Type new task")
                                    .border_style(Style::default().fg(Color::Green)),
                            )
                            .alignment(Alignment::Left);

                        frame.render_widget(input_block, chunks[1]);
                    }
                    _ => (),
                }

                // // Debug application state
                //
                // let paragraph = Paragraph::new(format!(
                //     "{}",
                //     match *app.stage.lock().unwrap() {
                //         AppStage::CreateNewItem => format!("new item: {}", app.new_item_name),
                //         AppStage::Default => "default".to_string(),
                //     }
                // ))
                // .block(create_block("App stage"))
                // .alignment(Alignment::Left);

                // frame.render_widget(paragraph, chunks[2]);
            })
            .expect("Terminal draw failed");

        match key_down_handler(&key_events_receiver, &mut app, &mut terminal) {
            true => break Result::Ok(()),
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
                    _ => {
                        sender.send(TerminalEvent::Input(Key::Char('q'))).unwrap();
                        break;
                    }
                },
                key => sender.send(TerminalEvent::Input(key)).unwrap(),
            }
        }
    });

    receiver
}

fn key_down_handler(
    receiver: &Receiver<TerminalEvent>,
    app: &mut App,
    terminal: &mut Terminal<TermionBackend<RawTerminal<Stdout>>>,
) -> bool {
    match receiver.recv() {
        Result::Ok(event) => key_action_mapper(event, app, terminal),
        Err(_) => {
            return true;
        }
    };

    false
}

fn key_action_mapper(
    event: TerminalEvent,
    app: &mut App,
    terminal: &mut Terminal<TermionBackend<RawTerminal<Stdout>>>,
) -> bool {
    match event {
        TerminalEvent::Input(Key::Char(key)) => match app.get_stage_clone() {
            AppStage::CreateNewItem => match key {
                '\n' => {
                    app.add_new_item();
                    app.reset_new_item_name();
                    app.set_stage(AppStage::Default);
                }
                key => app.new_item_add_character(key),
            },
            AppStage::Filter => match key {
                '\n' => {
                    app.set_stage(AppStage::Default);
                }
                key => app.filter_term_add_character(key),
            },
            AppStage::Default => match key {
                'n' => app.set_stage(AppStage::CreateNewItem),
                'f' => app.set_stage(AppStage::Filter),
                'd' => app.remove_task(),
                ' ' | '\n' => app.toggle_task(),
                's' => app.toggle_sorting(),
                'q' => {
                    terminal.clear().unwrap();
                    dump(
                        PATH_TO_FILE.to_string(),
                        Data {
                            items: app.list.items.clone(),
                        },
                    );
                    return true;
                }
                _ => (),
            },
        },
        TerminalEvent::Input(special_key) => match app.get_stage_clone() {
            AppStage::CreateNewItem => match special_key {
                Key::Backspace => app.new_item_remove_character(),
                _ => (),
            },
            AppStage::Filter => match special_key {
                Key::Backspace => app.filter_term_remove_character(),
                _ => (),
            },
            AppStage::Default => match special_key {
                Key::Backspace => app.remove_task(),
                Key::Down => app.list.next(),
                Key::Up => app.list.previous(),
                _ => (),
            },
        },
    };

    false
}

fn get_app_data() -> Vec<TodoItem> {
    let file = fs::read_to_string(PATH_TO_FILE).expect("Unable to read file");
    let data: Data = serde_json::from_str(file.as_str()).expect("Parsing json has failed");

    data.items
}
