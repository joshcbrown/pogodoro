use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, BorderType, Borders, ListState, Paragraph};
use tui::Frame;

struct Task {
    description: String,
}

pub struct TasksState {
    tasks: Vec<Task>,
    input: String,
    list_state: ListState,
    input_state: InputState,
}

pub enum InputState {
    Editing,
    Normal,
}

impl TasksState {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            input: String::new(),
            list_state: ListState::default(),
            input_state: InputState::Normal,
        }
    }
}

impl TasksState {
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .margin(2)
            .split(frame.size());

        let input = Block::default()
            .title("Add a task")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let tasks = Block::default()
            .title("Task list")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        frame.render_widget(input, chunks[0]);
        frame.render_widget(tasks, chunks[1]);
    }
}
