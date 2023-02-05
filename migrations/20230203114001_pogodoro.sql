CREATE TABLE tasks (
  desc TEXT PRIMARY KEY,
  task_dur INTEGER NOT NULL,
  short_break_dur INTEGER NOT NULL,
  long_break_dur INTEGER NOT NULL,
  completed INTEGER DEFAULT 0
);
