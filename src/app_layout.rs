use crate::app::AppStage;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub struct ListLayout<'a> {
    pub layout: Layout,
    pub new_item_input_block: Block<'a>,
    pub list_block: Block<'a>,
}

impl<'a> ListLayout<'a> {
    pub fn new() -> Self {
        Self {
            layout: Layout::default(),
            new_item_input_block: Block::default(),
            list_block: Block::default(),
        }
    }

    pub fn update_layout_chunks(&mut self, stage: &AppStage, area: Rect) -> Vec<Rect> {
        let constraint: Vec<Constraint> = match stage {
            AppStage::CreateNewItem => vec![Constraint::Percentage(50), Constraint::Percentage(50)],
            _ => vec![Constraint::Percentage(100)],
        };

        let border_color = match stage {
            AppStage::Default => Color::Green,
            _ => Color::Reset,
        };

        self.list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title("My tasks");

        let border_color = match stage {
            AppStage::CreateNewItem => Color::Green,
            _ => Color::Reset,
        };

        self.new_item_input_block = Block::default()
            .borders(Borders::ALL)
            .title("Type new task")
            .border_style(Style::default().fg(border_color));

        self.layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraint.as_ref());

        self.layout.split(area)
    }

    pub fn get_list_widget(items: Vec<ListItem<'a>>, block: Block<'a>) -> List<'a> {
        List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .block(block)
    }

    pub fn get_new_item_widget(item_name: &str, block: Block<'a>) -> Paragraph<'a> {
        Paragraph::new(format!("{}", item_name))
            .block(block)
            .alignment(Alignment::Left)
    }
}

struct AppLayout<'a> {
    layout: Layout,
    filter_block: Block<'a>,
    list_layout: ListLayout<'a>,
    info_block: Block<'a>,
    help_block: Block<'a>,
}

impl<'a> AppLayout<'a> {
    pub fn new() -> Self {
        Self {
            layout: Layout::default().direction(Direction::Vertical),
            filter_block: Block::default(),
            info_block: Block::default(),
            help_block: Block::default(),
            list_layout: ListLayout::new(),
        }
    }
}
