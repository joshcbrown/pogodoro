use std::fmt;
use std::time::Duration;
use std::time::Instant;
use tui::style::{Color, Style};

#[derive(Debug)]
pub struct Timer {
    start_time: Instant,
    dur: Duration,
    elapsed: Duration,
}

impl Timer {
    fn new(dur: Duration) -> Self {
        Self {
            dur,
            start_time: Instant::now(),
            elapsed: Duration::from_secs(0),
        }
    }

    fn update(&mut self) {
        // might check here later if timer is over
        self.elapsed = self.start_time.elapsed();
    }

    pub fn is_finished(&self) -> bool {
        // println!("{:#?} {:#?}", self.total_time, self.elapsed);
        self.elapsed >= self.dur
    }
}

impl fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_finished() {
            write!(f, "Finished!")
        } else {
            let to_go = (self.dur - self.elapsed + Duration::from_secs(1)).as_secs();
            write!(f, "{}m{}s", to_go / 60, to_go % 60)
        }
    }
}

#[derive(Debug)]
pub enum PomodoroState {
    Work,
    ShortBreak,
    LongBreak,
}

impl fmt::Display for PomodoroState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Work => "Work",
                Self::ShortBreak => "Short Break",
                Self::LongBreak => "Long Break",
            }
        )
    }
}

#[derive(Debug)]
pub struct Pomodoro {
    pub current: Timer,
    work_dur: Duration,
    short_break_dur: Duration,
    long_break_dur: Duration,
    state: PomodoroState,
    pomos_completed: u16,
}

impl Pomodoro {
    pub fn new(work_dur: Duration, short_break_dur: Duration, long_break_dur: Duration) -> Self {
        let mut first_timer = Timer::new(work_dur);
        first_timer.update();
        Self {
            current: first_timer,
            state: PomodoroState::Work,
            pomos_completed: 0,
            work_dur,
            short_break_dur,
            long_break_dur,
        }
    }

    pub fn update(&mut self) {
        self.current.update();
        if self.current.is_finished() {
            (self.state, self.current) = match self.state {
                PomodoroState::Work => {
                    self.pomos_completed += 1;
                    if self.pomos_completed % 4 == 0 {
                        (PomodoroState::LongBreak, Timer::new(self.long_break_dur))
                    } else {
                        (PomodoroState::ShortBreak, Timer::new(self.short_break_dur))
                    }
                }
                PomodoroState::ShortBreak | PomodoroState::LongBreak => {
                    (PomodoroState::Work, Timer::new(self.work_dur))
                }
            };
            self.current.update();
        }
    }

    pub fn style(&self) -> Style {
        match self.state {
            PomodoroState::Work => Style::default().fg(Color::Red),
            PomodoroState::ShortBreak => Style::default().fg(Color::Green),
            PomodoroState::LongBreak => Style::default().fg(Color::Blue),
        }
    }

    pub fn pomos_completed(&self) -> u16 {
        self.pomos_completed
    }

    pub fn state(&self) -> &PomodoroState {
        &self.state
    }
}
