use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Spans;
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use tui::Frame;
use unicode_width::UnicodeWidthStr;

struct Task {
    description: String,
}

pub struct TasksState {
    tasks: StatefulList<Task>,
    input: String,
    input_state: InputState,
}

pub enum InputState {
    Insert,
    Normal,
}

impl TasksState {
    pub fn new() -> Self {
        Self {
            tasks: StatefulList::new(),
            input: String::new(),
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

        let input_text = Paragraph::new(self.input.as_ref())
            .style(match self.input_state {
                InputState::Insert => Style::default().fg(Color::Yellow),
                InputState::Normal => Style::default(),
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Add a task"),
            );

        let task_list: Vec<ListItem> = self
            .tasks
            .items
            .iter()
            .map(|task| ListItem::new(vec![Spans::from(&task.description[..])]))
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

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match self.input_state {
            InputState::Normal => match key.code {
                KeyCode::Char('i') => self.input_state = InputState::Insert,
                KeyCode::Down | KeyCode::Char('j') => self.tasks.next(),
                KeyCode::Up | KeyCode::Char('k') => self.tasks.previous(),
                _ => {}
            },
            InputState::Insert => {
                match key.code {
                    KeyCode::Char(c) => self.input.push(c),
                    KeyCode::Esc => self.input_state = InputState::Normal,
                    KeyCode::Enter => self.tasks.items.push(Task {
                        description: self.input.drain(..).collect(),
                    }),
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    _ => {}
                }
                if key.code == KeyCode::Char('u') && key.modifiers == KeyModifiers::CONTROL {
                    self.input = String::new()
                };
            }
        }
    }
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn new() -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
