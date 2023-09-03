use crate::{
    db,
    pomodoro::centered_rect,
    states::{AppMessage, AppResult},
};

use chrono::{Duration, Local, NaiveDateTime};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use std::iter::repeat;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{
        block::Title, BarChart, Block, BorderType, Borders, Cell, Clear, Paragraph,
        Row as TableRow, Table, TableState,
    },
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
    pub completed: Option<NaiveDateTime>,
}

impl ToString for Task {
    fn to_string(&self) -> String {
        // we only call this function when looking at tasks
        // in the DB, so unwrapping is ok
        format!(
            "{:>3}: {} || {}/{}/{}",
            self.id.unwrap(),
            self.desc.as_ref().unwrap(),
            Self::format_time(self.work_secs),
            Self::format_time(self.short_break_secs),
            Self::format_time(self.long_break_secs),
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
            completed: None,
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

impl Task {
    fn format_time(seconds: u64) -> String {
        let mins = seconds / 60;
        let secs = seconds % 60;

        if secs == 0 {
            format!("{}m", mins)
        } else {
            format!("{}m{}s", mins, secs)
        }
    }

    fn to_table_row(&self) -> TableRow {
        let cells = [
            // TODO: come back and fix this unwrap when checking is done on task input
            Cell::from(self.desc.clone().unwrap()),
            Cell::from(Self::format_time(self.work_secs)),
            Cell::from(Self::format_time(self.short_break_secs)),
            Cell::from(Self::format_time(self.long_break_secs)),
        ];
        TableRow::new(cells)
    }
}

pub struct TasksState {
    task_tables: TaskTableGroup,
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
        let (incomplete, complete): (Vec<_>, Vec<_>) =
            tasks.into_iter().partition(|t| t.completed.is_none());
        let (new, in_progress): (Vec<_>, Vec<_>) =
            incomplete.into_iter().partition(|t| t.pomos_finished == 0);
        let last_day_complete: Vec<_> = complete
            .into_iter()
            .filter(|t| {
                Local::now()
                    .naive_local()
                    .signed_duration_since(t.completed.unwrap())
                    <= Duration::hours(24)
            })
            .collect();
        let task_tables = TaskTableGroup::new(vec![
            (new, "New".into()),
            (in_progress, "In Progress".into()),
            (last_day_complete, "Completed in the last day".into()),
        ]);

        let cycles: Vec<_> = crate::db::last_n_day_cycles(30)
            .await?
            .iter()
            .map(|(date, i)| (date.format("%d/%m").to_string(), *i))
            .collect();

        Ok(Self {
            task_tables,
            input: TaskInput::default(),
            input_state: InputState::Normal,
            cycles,
        })
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Percentage(30)])
            .margin(1)
            .split(frame.size());

        self.task_tables.render_on(frame, chunks[0]);
        self.render_barchart(frame, chunks[1]);

        match self.input_state {
            InputState::Insert => self.input.render_on(frame),
            InputState::Help => self.render_help(frame),
            _ => {}
        }
    }

    fn render_barchart<B: Backend>(&mut self, frame: &mut Frame<'_, B>, chunk: Rect) {
        let data: Vec<_> = self
            .cycles
            .iter()
            .rev()
            .take(chunk.width as usize / 10)
            .rev()
            .map(|(date, i)| (date.as_ref(), *i as u64))
            .collect();

        let barchart = BarChart::default()
            .block(
                Block::default()
                    .title(Title::from("Pomos over time").alignment(Alignment::Center))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .data(&data)
            .bar_width(9)
            .bar_style(Style::default().fg(Color::Yellow))
            .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));

        frame.render_widget(barchart, chunk);
    }

    fn render_help<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
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

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<AppMessage> {
        match self.input_state {
            InputState::Normal => match key.code {
                KeyCode::Char('?') => self.input_state = InputState::Help,
                KeyCode::Char('q') => return Ok(AppMessage::Finish),
                KeyCode::Char('i') => {
                    self.task_tables.focused = None;
                    self.input_state = InputState::Insert;
                    self.input.next()
                }
                // allow user to complete task
                KeyCode::Char('c') => {
                    if let Some(task) = self.task_tables.selected() {
                        db::complete(task.id.unwrap() as i64).await?;
                        *self = Self::new().await?;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => self.task_tables.next_task(),
                KeyCode::Up | KeyCode::Char('k') => self.task_tables.prev_task(),
                KeyCode::Tab | KeyCode::Char('l') => self.task_tables.next(),
                KeyCode::BackTab | KeyCode::Char('h') => self.task_tables.previous(),
                KeyCode::Enter => {
                    if let Some(task) = self.task_tables.selected() {
                        return Ok(AppMessage::Begin(task.clone()));
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
                        self.task_tables.add_task(new_task)
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
        Ok(AppMessage::DoNothing)
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

impl Focus for InputGroup {
    fn len(&self) -> usize {
        self.inputs.len()
    }

    fn empty(&self) -> bool {
        self.inputs.is_empty()
    }

    fn focus(&mut self) -> &mut Option<usize> {
        &mut self.focused
    }
}

impl InputGroup {
    // render the group on a frame
    pub fn render_on<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        if self.inputs.is_empty() {
            return;
        }

        let height = self.inputs.len() * 3 + 2;
        let width = std::cmp::max(50, frame.size().width / 3);
        let outer_rect = centered_rect(width, height as u16, frame.size());
        let outer_block = Block::default().title("Create task").borders(Borders::ALL);
        let rect = outer_block.inner(outer_rect);
        frame.render_widget(Clear, outer_rect);
        frame.render_widget(outer_block, outer_rect);

        let sub_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                repeat(Constraint::Length(3))
                    .take(self.inputs.len())
                    .collect::<Vec<Constraint>>(),
            )
            .split(rect);

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

    // add a char to focused input
    fn push(&mut self, c: char) {
        if self.focused.is_some() {
            self.inputs[self.focused.unwrap()].push(c)
        }
    }

    // pop a char from focused input
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

trait Focus {
    fn focus(&mut self) -> &mut Option<usize>;
    fn empty(&self) -> bool;
    fn len(&self) -> usize;
    fn pre_move(&mut self) {}

    fn move_focus<F: Fn(usize) -> usize>(&mut self, f: F) {
        let empty = self.empty();
        let focus = self.focus();
        if focus.is_some() {
            *focus = focus.map(f);
            return;
        }
        *focus = if empty { None } else { Some(0) }
    }

    fn next(&mut self) {
        self.pre_move();
        let len = self.len();
        self.move_focus(|n| (n + 1) % len);
    }

    fn previous(&mut self) {
        self.pre_move();
        let len = self.len();
        self.move_focus(|n| if n == 0 { len - 1 } else { n - 1 })
    }
}

impl Focus for TaskInput {
    fn empty(&self) -> bool {
        self.0.empty()
    }

    fn focus(&mut self) -> &mut Option<usize> {
        self.0.focus()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl TaskInput {
    const DEFAULT_SECS: (u64, u64, u64) = (25 * 60, 5 * 60, 15 * 60);
    // HACK: this is kinda inheritance but not sure what else I should do
    pub fn render_on<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        self.0.render_on(frame)
    }

    fn push(&mut self, c: char) {
        self.0.push(c)
    }

    fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }

    fn clear(&mut self) {
        self.0.clear()
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
        let work_secs = self.parse_secs(1, Self::DEFAULT_SECS.0);
        let short_break_secs = self.parse_secs(2, Self::DEFAULT_SECS.1);
        let long_break_secs = self.parse_secs(3, Self::DEFAULT_SECS.2);
        (
            self.0.inputs[0].text.drain(..).collect(),
            work_secs,
            short_break_secs,
            long_break_secs,
        )
    }
}

struct TaskTableGroup {
    tables: Vec<TaskTable>,
    focused: Option<usize>,
}

impl Focus for TaskTableGroup {
    fn len(&self) -> usize {
        self.tables.len()
    }

    fn focus(&mut self) -> &mut Option<usize> {
        &mut self.focused
    }

    fn empty(&self) -> bool {
        self.tables.is_empty()
    }

    fn pre_move(&mut self) {
        if self.focused.is_some() {
            self.tables[self.focused.unwrap()].state.select(None)
        }
    }
}

impl TaskTableGroup {
    fn new(tasks: Vec<(Vec<Task>, String)>) -> Self {
        Self {
            tables: tasks
                .into_iter()
                .map(|(tasks, title)| TaskTable::new(tasks, title))
                .collect(),
            focused: None,
        }
    }

    fn next_task(&mut self) {
        if self.focused.is_none() {
            self.next()
        }
        self.tables[self.focused.unwrap()].next()
    }

    fn prev_task(&mut self) {
        if self.focused.is_none() {
            self.next()
        }
        self.tables[self.focused.unwrap()].previous()
    }

    fn render_on<B: Backend>(&mut self, frame: &mut Frame<'_, B>, chunk: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(chunk);

        for (i, (table, &sub_chunk)) in self.tables.iter_mut().zip(chunks.iter()).enumerate() {
            table.render_on(frame, sub_chunk, i == self.focused.unwrap_or(usize::MAX))
        }
    }

    fn selected(&self) -> Option<&Task> {
        self.tables[self.focused?].selected()
    }

    fn add_task(&mut self, task: Task) {
        self.tables[0].tasks.push(task)
    }
}

#[derive(Default)]
struct TaskTable {
    state: TableState,
    title: String,
    tasks: Vec<Task>,
}

impl TaskTable {
    fn new(tasks: Vec<Task>, title: String) -> Self {
        TaskTable {
            tasks,
            title,
            ..Default::default()
        }
    }

    fn move_focus<F: Fn(usize) -> usize>(&mut self, f: F) {
        let selected = self.state.selected();
        let new_selected = if selected.is_some() {
            selected.map(f)
        } else if !self.tasks.is_empty() {
            Some(0)
        } else {
            None
        };
        self.state.select(new_selected)
    }

    fn next(&mut self) {
        let len = self.tasks.len();
        self.move_focus(|i| (i + 1) % len)
    }

    fn previous(&mut self) {
        let len = self.tasks.len();
        self.move_focus(|i| if i == 0 { len - 1 } else { i - 1 })
    }

    fn selected(&self) -> Option<&Task> {
        Some(&self.tasks[self.state.selected()?])
    }

    pub fn render_on<B: Backend>(&mut self, frame: &mut Frame<'_, B>, chunk: Rect, focused: bool) {
        let task_list = self.tasks.iter().map(|task| task.to_table_row());

        let header_cells = ["Task", "Work", "Short break", "Long break"]
            .iter()
            .map(|&h| {
                Cell::from(Text::styled(
                    h,
                    Style::default()
                        .fg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::ITALIC),
                ))
            });

        let header = TableRow::new(header_cells)
            .bottom_margin(1)
            .style(Style::default()); //.add_modifier(Modifier::UNDERLINED));
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let task_list = Table::new(task_list)
            .header(header)
            .block(
                Block::default()
                    .title(Title::from(self.title.clone()).alignment(Alignment::Center))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Red))
            .widths(&[
                Constraint::Percentage(50),
                Constraint::Percentage(16),
                Constraint::Percentage(17),
                Constraint::Percentage(17),
            ]);

        frame.render_stateful_widget(task_list, chunk, &mut self.state);
    }
}
