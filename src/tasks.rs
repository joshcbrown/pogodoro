use crate::{db, pomodoro::centered_rect, states::AppResult};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use std::iter::repeat;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{BarChart, Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Debug)]
pub struct Task {
    pub id: Option<u32>,
    pub desc: Option<String>,
    pub work_secs: u64,
    pub short_break_secs: u64,
    pub long_break_secs: u64,
    pub pomos_finished: u32,
    pub completed: bool,
}

impl ToString for Task {
    fn to_string(&self) -> String {
        // we only call this function when looking at tasks
        // in the DB, so unwrapping is ok
        format!(
            "{:>3}: {} || {}/{}/{}",
            self.id.unwrap(),
            self.desc.as_ref().unwrap(),
            self.work_secs / 60,
            self.short_break_secs / 60,
            self.long_break_secs / 60
        )
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: None,
            desc: None,
            work_secs: 25 * 60,
            short_break_secs: 5 * 60,
            long_break_secs: 15 * 60,
            pomos_finished: 0,
            completed: false,
        }
    }
}

impl FromRow<'_, SqliteRow> for Task {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        // think all these unwraps are ok because these values
        // will never be high enough to panic on conversion
        // (unless you try really really hard)
        Ok(Self {
            id: Some(row.try_get("id")?),
            desc: row.try_get("desc")?,
            work_secs: row.try_get::<i64, &str>("work_secs")?.try_into().unwrap(),
            short_break_secs: row
                .try_get::<i64, &str>("short_break_secs")?
                .try_into()
                .unwrap(),
            long_break_secs: row
                .try_get::<i64, &str>("long_break_secs")?
                .try_into()
                .unwrap(),
            pomos_finished: row
                .try_get::<i64, &str>("pomos_finished")?
                .try_into()
                .unwrap(),
            completed: row.try_get("completed")?,
        })
    }
}

pub struct TasksState {
    tasks: StatefulList<Task>,
    input: TaskInput,
    cycles: Vec<(String, usize)>,
    input_state: InputState,
}

pub enum InputState {
    Insert,
    Normal,
    Help,
}

const HELP_TEXT: &str = "This screen has two modes: insert, and normal.
The user is in insert mode when they are filling in a new task's
fields at the top of the screen.
The user is in normal mode when they are selecting a task to begin. 
The app begins in normal mode.

Use [tab] or [i] to enter insert mode,
[tab] to switch between fields, and [enter] to submit the task.
Use [esc] to exit insert mode into normal mode.

While in normal mode, use [j], [k], [up], and [down]
to navigate task entries in the main box.
Use [enter] to select a task and begin a pomodoro for it.
You can also exit the program from normal mode with [q] or [esc].

Use [?] to quit this help message into normal mode.";

impl TasksState {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let tasks = crate::db::read_tasks().await?;

        let cycles: Vec<_> = crate::db::last_n_day_cycles(30)
            .await?
            .iter()
            .map(|(date, i)| (date.format("%d/%m").to_string(), *i))
            .collect();

        Ok(Self {
            tasks: StatefulList {
                items: tasks,
                ..Default::default()
            },
            input: TaskInput::default(),
            input_state: InputState::Normal,
            cycles,
        })
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Percentage(20),
            ])
            .margin(2)
            .split(frame.size());

        let task_list: Vec<ListItem> = self
            .tasks
            .items
            .iter()
            .map(|task| ListItem::new(vec![Line::from(task.to_string())]))
            .collect();

        let task_list = List::new(task_list)
            .block(
                Block::default()
                    .title("Task list")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::DarkGray)
                    .bg(Color::LightBlue),
            );

        self.input.render_on(frame, chunks[0]);
        frame.render_stateful_widget(task_list, chunks[1], &mut self.tasks.state);

        let data: Vec<_> = self
            .cycles
            .iter()
            .rev()
            .take(chunks[2].width as usize / 10)
            .rev()
            .map(|(date, i)| (date.as_ref(), *i as u64))
            .collect();

        let barchart = BarChart::default()
            .block(
                Block::default()
                    .title("Pomos over time")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .data(&data)
            .bar_width(9)
            .bar_style(Style::default().fg(Color::Yellow))
            .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));

        frame.render_widget(barchart, chunks[2]);

        if let InputState::Help = &self.input_state {
            // hard coded vals for text width and height
            let help_chunk = centered_rect(70, 18, frame.size());

            let help_text = Paragraph::new(HELP_TEXT)
                .block(
                    Block::default()
                        .title("Help")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(Clear, help_chunk);
            frame.render_widget(help_text, help_chunk);
        }
    }

    pub fn should_finish(&self, key: &KeyEvent) -> bool {
        if let InputState::Normal = self.input_state {
            key.code == KeyCode::Char('q')
        } else {
            false
        }
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<Option<Task>> {
        match self.input_state {
            InputState::Normal => match key.code {
                KeyCode::Char('?') => self.input_state = InputState::Help,
                KeyCode::Char('i') | KeyCode::Tab => {
                    self.tasks.state.select(None);
                    self.input_state = InputState::Insert;
                    self.input.next()
                }
                // allow user to complete task
                KeyCode::Char('c') => {
                    if let Some(task) = self.tasks.selected() {
                        db::complete(task.id.unwrap() as i64).await?;
                        *self = Self::new().await?;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => self.tasks.next(),
                KeyCode::Up | KeyCode::Char('k') => self.tasks.previous(),
                KeyCode::Enter => {
                    if let Some(n) = self.tasks.state.selected() {
                        return Ok(Some(self.tasks.items[n].clone()));
                    }
                }
                _ => {}
            },
            InputState::Insert => {
                match key.code {
                    KeyCode::Char(c) => self.input.push(c),
                    KeyCode::Esc => {
                        self.input_state = InputState::Normal;
                        self.input.0.focused = None
                    }
                    KeyCode::Tab => self.input.next(),
                    KeyCode::BackTab => self.input.previous(),
                    KeyCode::Enter => {
                        let (desc, work_secs, sb_secs, lb_secs) = self.input.get_task();
                        let new_task = db::write_and_return_task(desc, work_secs, sb_secs, lb_secs)
                            .await
                            .unwrap();
                        self.tasks.items.push(new_task)
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    _ => {}
                }
                if key.code == KeyCode::Char('u') && key.modifiers == KeyModifiers::CONTROL {
                    self.input.clear()
                };
            }
            InputState::Help => {
                if key.code == KeyCode::Char('?') {
                    self.input_state = InputState::Normal
                }
            }
        };
        Ok(None)
    }
}

struct UserInput {
    title: String,
    text: String,
}

impl UserInput {
    fn new(title: String) -> Self {
        Self {
            title,
            text: String::new(),
        }
    }

    fn to_widget(&self, focused: Option<bool>) -> Paragraph {
        Paragraph::new(self.text.clone())
            .style(if let Some(true) = focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(self.title.as_str()),
            )
    }

    fn width(&self) -> usize {
        self.text.width()
    }

    fn pop(&mut self) -> Option<char> {
        self.text.pop()
    }

    fn push(&mut self, c: char) {
        self.text.push(c)
    }
}

#[derive(Default)]
pub struct InputGroup {
    inputs: Vec<UserInput>,
    focused: Option<usize>,
}

impl InputGroup {
    pub fn render_on<B: Backend>(&mut self, frame: &mut Frame<'_, B>, chunk: Rect) {
        if self.inputs.is_empty() {
            return;
        }

        let percentage = Constraint::Percentage(100 / self.inputs.len() as u16);
        let sub_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                repeat(percentage)
                    .take(self.inputs.len())
                    .collect::<Vec<Constraint>>(),
            )
            .split(chunk);

        for (i, (input, sub_chunk)) in self.inputs.iter().zip(sub_chunks.iter()).enumerate() {
            frame.render_widget(input.to_widget(self.focused.map(|j| i == j)), *sub_chunk)
        }

        if self.focused.is_none() {
            return;
        }
        let focused_idx = self.focused.unwrap();
        let focused_input = &self.inputs[focused_idx];

        frame.set_cursor(
            sub_chunks[focused_idx].x + focused_input.width() as u16 + 1,
            sub_chunks[focused_idx].y + 1,
        )
    }

    fn move_focus<F: Fn(usize) -> usize>(&mut self, f: F) {
        if self.focused.is_some() {
            self.focused = self.focused.map(f);
            return;
        }
        self.focused = if !self.inputs.is_empty() {
            Some(0)
        } else {
            None
        }
    }

    fn next(&mut self) {
        let len = self.inputs.len();
        self.move_focus(|n| (n + 1) % len)
    }

    fn previous(&mut self) {
        let len = self.inputs.len();
        self.move_focus(|n| if n == 0 { len - 1 } else { n - 1 })
    }

    fn push(&mut self, c: char) {
        if self.focused.is_some() {
            self.inputs[self.focused.unwrap()].push(c)
        }
    }

    fn pop(&mut self) -> Option<char> {
        let idx = self.focused?;
        self.inputs[idx].pop()
    }

    fn clear(&mut self) {
        if self.focused.is_some() {
            self.inputs[self.focused.unwrap()].text = String::new()
        }
    }
}

struct TaskInput(InputGroup);

impl Default for TaskInput {
    fn default() -> Self {
        Self(InputGroup {
            inputs: vec![
                UserInput::new("Task name".into()),
                UserInput::new("Work duration (m)".into()),
                UserInput::new("Short break duration (m)".into()),
                UserInput::new("Long break duration (m)".into()),
            ],
            focused: None,
        })
    }
}

const DEFAULT_SECS: (u64, u64, u64) = (25 * 60, 5 * 60, 15 * 60);

impl TaskInput {
    // HACK: this is kinda inheritance but not sure what else I should do
    pub fn render_on<B: Backend>(&mut self, frame: &mut Frame<'_, B>, chunk: Rect) {
        self.0.render_on(frame, chunk)
    }

    fn push(&mut self, c: char) {
        self.0.push(c)
    }

    fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }

    fn next(&mut self) {
        self.0.next()
    }

    fn clear(&mut self) {
        self.0.clear()
    }

    fn previous(&mut self) {
        self.0.previous()
    }

    fn parse_secs(&mut self, i: usize, default: u64) -> i64 {
        let text: &mut String = &mut self.0.inputs[i].text;
        (text
            .drain(..)
            .collect::<String>()
            .parse::<f64>()
            .unwrap_or((default / 60) as f64)
            * 60.0) as i64
    }

    fn get_task(&mut self) -> (String, i64, i64, i64) {
        let work_secs = self.parse_secs(1, DEFAULT_SECS.0);
        let short_break_secs = self.parse_secs(2, DEFAULT_SECS.1);
        let long_break_secs = self.parse_secs(3, DEFAULT_SECS.2);
        (
            self.0.inputs[0].text.drain(..).collect(),
            work_secs,
            short_break_secs,
            long_break_secs,
        )
    }
}

/// struct is a slightly cleaned up version of a struct in tui-rs's demo
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> Default for StatefulList<T> {
    fn default() -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn move_focus<F: Fn(usize) -> usize>(&mut self, f: F) {
        let selected = self.state.selected();
        let new_selected = if selected.is_some() {
            selected.map(f)
        } else if !self.items.is_empty() {
            Some(0)
        } else {
            None
        };
        self.state.select(new_selected)
    }

    pub fn next(&mut self) {
        let len = self.items.len();
        self.move_focus(|i| (i + 1) % len)
    }

    pub fn previous(&mut self) {
        let len = self.items.len();
        self.move_focus(|i| if i == 0 { len - 1 } else { i - 1 })
    }

    pub fn selected(&self) -> Option<&T> {
        Some(&self.items[self.state.selected()?])
    }
}
