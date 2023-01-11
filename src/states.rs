use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io::Stderr;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::widgets::{Block, BorderType, Borders, ListState, Paragraph};
use tui::Frame;

use crate::app::{AppResult, AppState};
use crate::pomodoro::Pomodoro;

pub struct WorkingState {
    pomo: Pomodoro,
}

impl WorkingState {
    pub fn new(working_dur: Duration, short_break_dur: Duration, long_break_dur: Duration) -> Self {
        let pomo = Pomodoro::new(working_dur, short_break_dur, long_break_dur);
        Self { pomo }
    }
}

const POMO_HEIGHT: u16 = 6;
const POMO_WIDTH: u16 = 25;

impl AppState for WorkingState {
    fn render(&mut self, frame: &mut Frame<'_, CrosstermBackend<Stderr>>) {
        let frame_rect = frame.size();
        let vert_buffer = frame_rect.height.checked_sub(POMO_HEIGHT).unwrap_or(0) / 2;
        let hor_buffer = frame_rect.width.checked_sub(POMO_WIDTH).unwrap_or(0) / 2;

        let vert_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(vert_buffer),
                Constraint::Length(POMO_HEIGHT),
                Constraint::Min(vert_buffer),
            ])
            .split(frame.size());

        let hor_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(hor_buffer),
                Constraint::Length(POMO_WIDTH),
                Constraint::Min(hor_buffer),
            ])
            .split(vert_chunks[1]);

        let pomo_widget = Paragraph::new(format!(
            "Remaining: {}\nFinished: {}\n\n[q]uit",
            self.pomo.current,
            self.pomo.pomos_completed()
        ))
        .block(
            Block::default()
                .title(format!("Pogodoro — {}", self.pomo.state()))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(self.pomo.style()),
        )
        .alignment(Alignment::Center);

        frame.render_widget(pomo_widget, hor_chunks[1]);
    }

    fn tick(&mut self) {
        self.pomo.update()
    }

    fn handle_key_event(&mut self, _key: KeyEvent) {}
}

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

impl AppState for TasksState {
    fn tick(&mut self) {}

    fn render(&mut self, frame: &mut Frame<'_, CrosstermBackend<Stderr>>) {
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

    fn handle_key_event(&mut self, _key: KeyEvent) {}
}
