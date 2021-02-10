use crate::app::AppStage;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use tui::Frame;

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

    pub fn draw_list_widget<B>(
        &self,
        frame: &mut Frame<B>,
        items: Vec<ListItem<'a>>,
        area: Rect,
        state: &mut ListState,
    ) where
        B: Backend,
    {
        frame.render_stateful_widget(
            self.get_list_widget(items, self.list_block.clone()),
            area,
            state,
        );
    }

    fn get_list_widget(&self, items: Vec<ListItem<'a>>, block: Block<'a>) -> List<'a> {
        List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .block(block)
    }

    pub fn draw_new_item_widget<B>(&self, frame: &mut Frame<B>, item_name: &str, area: Rect)
    where
        B: Backend,
    {
        frame.render_widget(
            self.get_new_item_widget(item_name, self.new_item_input_block.clone()),
            area,
        );
    }

    fn get_new_item_widget(&self, item_name: &str, block: Block<'a>) -> Paragraph<'a> {
        Paragraph::new(format!("{}", item_name))
            .block(block)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left)
    }
}

pub struct AppLayout<'a> {
    pub layout: Layout,
    pub filter_block: Block<'a>,
    pub list_layout: ListLayout<'a>,
    pub info_block: Block<'a>,
    pub help_block: Paragraph<'a>,
}

impl<'a> AppLayout<'a> {
    pub fn new() -> Self {
        Self {
            layout: Layout::default(),
            filter_block: Block::default(),
            info_block: Block::default(),
            help_block: Paragraph::new(""),
            list_layout: ListLayout::new(),
        }
    }

    pub fn update_layout_chunks(&mut self, stage: &AppStage, area: Rect) -> (Vec<Rect>, Vec<Rect>) {
        let constraint: Vec<Constraint> = match stage {
            _ => vec![
                Constraint::Length(3),
                Constraint::Min(4),
                Constraint::Length(3),
            ],
        };

        self.layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraint.as_ref());

        let border_color = match stage {
            AppStage::Filter => Color::Green,
            _ => Color::Reset,
        };

        self.filter_block = Block::default()
            .borders(Borders::ALL)
            .title("Filter")
            .border_style(Style::default().fg(border_color));

        let help_block = Block::default().borders(Borders::ALL).title("Help");

        self.help_block = match stage {
            AppStage::Default => {
                Paragraph::new("q - quit, s - sort, n - new task, f - filter task")
                    .block(help_block)
            }
            AppStage::CreateNewItem => Paragraph::new("Enter - add item").block(help_block),
            AppStage::Filter => Paragraph::new("Enter - apply filter").block(help_block),
        };

        let app_layout_chunks = self.layout.split(area);
        let list_layout_chunks = match stage {
            _ => self
                .list_layout
                .update_layout_chunks(stage, app_layout_chunks[1]),
        };

        (app_layout_chunks, list_layout_chunks)
    }

    pub fn draw_help_widget<B>(&self, frame: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        frame.render_widget(self.help_block.clone(), area);
    }

    pub fn draw_filter_widget<B>(&self, frame: &mut Frame<B>, filter_term: &str, area: Rect)
    where
        B: Backend,
    {
        frame.render_widget(
            self.get_filter_widget(filter_term, self.filter_block.clone()),
            area,
        );
    }

    fn get_filter_widget(&self, filter_term: &str, block: Block<'a>) -> Paragraph<'a> {
        let text = if filter_term == "" {
            "None"
        } else {
            filter_term
        };

        Paragraph::new(format!("{}", text))
            .block(block)
            .alignment(Alignment::Left)
    }
}
