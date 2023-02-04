use std::iter::repeat;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use sqlx::{query, SqliteConnection, Connection};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Spans;
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use tui::Frame;
use unicode_width::UnicodeWidthStr;

use crate::states::AppState;

#[derive(Clone, Debug)]
pub struct Task {
    pub desc: Option<String>,
    pub work_dur: Duration,
    pub short_break_dur: Duration,
    pub long_break_dur: Duration,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            desc: None,
            work_dur: Duration::from_secs(60 * 25),
            short_break_dur: Duration::from_secs(60 * 5),
            long_break_dur: Duration::from_secs(60 * 15),
        }
    }
}

impl ToString for Task {
    fn to_string(&self) -> String {
        // TODO: investigate jankness
        format!(
            "{} || {}/{}/{}",
            self.desc.as_ref().unwrap(),
            self.work_dur.as_secs() / 60,
            self.short_break_dur.as_secs() / 60,
            self.long_break_dur.as_secs() / 60
        )
    }
}

pub struct TasksState {
    tasks: StatefulList<Task>,
    new_tasks: Vec<Task>,
    input: TaskInput,
    input_state: InputState,
}

pub enum InputState {
    Insert,
    Normal,
}

impl Default for TasksState {
    fn default() -> Self {
        Self {
            tasks: StatefulList::default(),
            new_tasks: Self::read_db().await,
            input: TaskInput::default(),
            input_state: InputState::Normal,
        }
    }
}

const TABLE_NAME: &'static str = "tasks";

impl TasksState {
    async fn read_db() -> Vec<Task> {
        let mut conn = SqliteConnection::connect(AppState::db_path().to_str().unwrap()).await.unwrap();
        query!("SELECT * FROM tasks WHERE completed = 0")
            .map(|task| Task {
                desc: Some(task.desc),
                work_dur: Duration::from_secs(task.task_dur as u64 * 60),
                short_break_dur: Duration::from_secs(task.short_break_dur as u64 * 60),
                long_break_dur: Duration::from_secs(task.long_break_dur as u64 * 60),
            })
            .fetch_all(&mut conn)
            .await
        .unwrap()
            
        // Vec::new()
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .margin(2)
            .split(frame.size());

        let task_list: Vec<ListItem> = self
            .tasks
            .items
            .iter()
            // unwrapping is ok here because the only way to be on this screen
            // is to have a valid description
            .map(|task| ListItem::new(vec![Spans::from(task.to_string())]))
            .collect();

        let task_list = List::new(task_list)
            .block(
                Block::default()
                    .title("Task list")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        self.input.render_on(frame, chunks[0]);
        frame.render_stateful_widget(task_list, chunks[1], &mut self.tasks.state);
    }

    pub fn should_finish(&self, key: &KeyEvent) -> bool {
        if let InputState::Normal = self.input_state {
            key.code == KeyCode::Char('q')
        } else {
            false
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<Task> {
        // TODO: tidy
        match self.input_state {
            InputState::Normal => match key.code {
                KeyCode::Char('i') => {
                    self.input_state = InputState::Insert;
                    self.input.0.next()
                }
                KeyCode::Down | KeyCode::Char('j') => self.tasks.next(),
                KeyCode::Up | KeyCode::Char('k') => self.tasks.previous(),
                KeyCode::Enter => {
                    if let Some(n) = self.tasks.state.selected() {
                        return Some(self.tasks.items[n].clone());
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
                    KeyCode::Tab => self.input.0.next(),
                    // TODO: only accept non-empty descs
                    KeyCode::Enter => self.tasks.items.push(self.input.get_task()),
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    _ => {}
                }
                if key.code == KeyCode::Char('u') && key.modifiers == KeyModifiers::CONTROL {
                    // TODO: fix this
                    self.input.0.clear()
                };
            }
        };
        None
    }
}

struct UserInput {
    title: String,
    text: String,
    input_state: InputState,
}

impl UserInput {
    fn new(title: String) -> Self {
        Self {
            title,
            text: String::new(),
            input_state: InputState::Normal,
        }
    }

    fn to_widget(&self) -> Paragraph {
        Paragraph::new(self.text.as_ref())
            .style(match self.input_state {
                InputState::Insert => Style::default().fg(Color::Yellow),
                InputState::Normal => Style::default(),
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

        for (input, sub_chunk) in self.inputs.iter().zip(&sub_chunks) {
            frame.render_widget(input.to_widget(), *sub_chunk)
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

    fn next(&mut self) {
        match self.focused {
            None => {
                self.focused = if !self.inputs.is_empty() {
                    Some(0)
                } else {
                    None
                }
            }
            Some(n) => self.focused = Some((n + 1) % self.inputs.len()),
        }
    }

    fn push(&mut self, c: char) {
        if self.focused.is_none() {
            return;
        }

        self.inputs[self.focused.unwrap()].push(c)
    }

    fn pop(&mut self) -> Option<char> {
        self.focused?;
        self.inputs[self.focused.unwrap()].pop()
    }

    fn clear(&mut self) {
        if self.focused.is_none() {
            return;
        }

        self.inputs[self.focused.unwrap()].text = String::new()
    }
}

struct TaskInput(InputGroup);

impl Default for TaskInput {
    fn default() -> Self {
        Self(InputGroup {
            inputs: vec![
                UserInput::new("Task name".into()),
                UserInput::new("Work duration".into()),
                UserInput::new("SB duration".into()),
                UserInput::new("LB duration".into()),
            ],
            focused: None,
        })
    }
}

impl TaskInput {
    pub fn render_on<B: Backend>(&mut self, frame: &mut Frame<'_, B>, chunk: Rect) {
        self.0.render_on(frame, chunk)
    }

    fn push(&mut self, c: char) {
        self.0.push(c)
    }

    fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }

    fn parse_dur(&mut self, i: usize, default: Duration) -> Duration {
        let text: &mut String = &mut self.0.inputs[i].text;
        if text.is_empty() {
            default
        } else {
            match text.drain(..).collect::<String>().parse::<u64>() {
                Ok(n) => Duration::from_secs(n * 60),
                Err(_) => default,
            }
        }
    }

    fn get_task(&mut self) -> Task {
        let default = Task::default();
        let work_dur = self.parse_dur(1, default.work_dur);
        let short_break_dur = self.parse_dur(2, default.short_break_dur);
        let long_break_dur = self.parse_dur(3, default.short_break_dur);
        Task {
            desc: Some(self.0.inputs[0].text.drain(..).collect()),
            work_dur,
            short_break_dur,
            long_break_dur,
        }
    }
}

/// struct courtesy of tui-rs's demo
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

    pub fn next(&mut self) {
        self.state.select(match self.state.selected() {
            Some(i) => Some((i + 1) % self.items.len()),
            None => {
                if !self.items.is_empty() {
                    Some(0)
                } else {
                    None
                }
            }
        })
    }

    pub fn previous(&mut self) {
        self.state.select(match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    Some(self.items.len() - 1)
                } else {
                    Some(i - 1)
                }
            }
            None => {
                if !self.items.is_empty() {
                    Some(0)
                } else {
                    None
                }
            }
        });
    }
}
