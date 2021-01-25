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
use tui::text::{Span, Spans};
use tui::widgets::ListItem;
use tui::Terminal;

use crate::app::{App, AppStage, TodoItem};
use crate::app_layout::AppLayout;
use std::path::PathBuf;

mod app;
mod app_layout;
mod utils;

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    items: Vec<TodoItem>,
}

fn dump(data: Data) {
    let (_path_to_file_dir, path_to_file) = get_file_path();

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
                let items: Vec<ListItem> = app
                    .get_filtered_items()
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
                    })
                    .collect();

                let mut app_layout = AppLayout::new();
                let frame_size = frame.size();

                let (app_chunks, list_chunks) =
                    app_layout.update_layout_chunks(&*app.stage.lock().unwrap(), frame_size);

                app_layout.draw_filter_widget(frame, &app.filter_term, app_chunks[0]);
                app_layout.list_layout.draw_list_widget(
                    frame,
                    items,
                    list_chunks[0],
                    &mut app.list.state,
                );
                app_layout.draw_help_widget(frame, app_chunks[2]);

                match &*app.stage.lock().unwrap() {
                    AppStage::CreateNewItem => {
                        app_layout.list_layout.draw_new_item_widget(
                            frame,
                            &app.new_item_name,
                            list_chunks[1],
                        );
                    }
                    _ => (),
                }
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
                    AppStage::CreateNewItem | AppStage::Filter => {
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
                    dump(Data {
                        items: app.list.items.clone(),
                    });
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
    let (path_to_file_dir, path_to_file) = get_file_path();

    match fs::read_dir(&path_to_file_dir) {
        Ok(_) => {}
        Err(_) => fs::create_dir_all(path_to_file_dir).unwrap(),
    }

    match fs::read_to_string(path_to_file) {
        Ok(data) => {
            let data: Data = serde_json::from_str(data.as_str()).expect("Parsing json has failed");
            data.items
        }
        Err(_) => vec![],
    }
}

fn get_file_path() -> (PathBuf, PathBuf) {
    let mut path_to_file = dirs::home_dir().unwrap();
    path_to_file.push(".rudo");
    let path_to_file_dir = path_to_file.clone();

    path_to_file.push("todos.json");

    (path_to_file_dir, path_to_file)
}
