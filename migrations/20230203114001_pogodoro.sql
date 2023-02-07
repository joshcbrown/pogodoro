CREATE TABLE tasks (
  desc TEXT,
  task_dur INTEGER NOT NULL,
  short_break_dur INTEGER NOT NULL,
  long_break_dur INTEGER NOT NULL,
  pomos_finished INTEGER NOT NULL DEFAULT 0,
  -- bool value
  completed INTEGER NOT NULL DEFAULT 0
);
