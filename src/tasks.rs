use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Spans;
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use tui::Frame;
use unicode_width::UnicodeWidthStr;

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

pub struct TasksState {
    tasks: StatefulList<Task>,
    input: UserInput,
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
            input: UserInput::new("Add a task".into()),
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

        let input_text = self.input.to_widget();

        let task_list: Vec<ListItem> = self
            .tasks
            .items
            .iter()
            .map(|task| ListItem::new(vec![Spans::from(task.desc.as_ref().unwrap().as_ref())]))
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

        frame.render_widget(input_text, chunks[0]);
        frame.render_stateful_widget(task_list, chunks[1], &mut self.tasks.state);

        if let InputState::Insert = self.input_state {
            frame.set_cursor(chunks[0].x + self.input.width() as u16 + 1, chunks[0].y + 1)
        }
    }

    pub fn should_finish(&self, key: &KeyEvent) -> bool {
        if let InputState::Normal = self.input_state {
            key.code == KeyCode::Char('q') || key.code == KeyCode::Esc
        } else {
            false
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<Task> {
        match self.input_state {
            InputState::Normal => match key.code {
                KeyCode::Char('i') => self.input_state = InputState::Insert,
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
                    KeyCode::Esc => self.input_state = InputState::Normal,
                    KeyCode::Enter => self.tasks.items.push(Task {
                        desc: Some(self.input.text.drain(..).collect()),
                        ..Task::default()
                    }),
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    _ => {}
                }
                if key.code == KeyCode::Char('u') && key.modifiers == KeyModifiers::CONTROL {
                    self.input.text = String::new()
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
            Some(i) => {
                if i >= self.items.len() - 1 {
                    Some(0)
                } else {
                    Some(i + 1)
                }
            }
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
