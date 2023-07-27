CREATE TABLE tasks (
  id INTEGER PRIMARY KEY,
  desc TEXT,
  work_secs INTEGER NOT NULL,
  short_break_secs INTEGER NOT NULL,
  long_break_secs INTEGER NOT NULL,
  pomos_finished INTEGER NOT NULL DEFAULT 0,
  -- bool value
  completed BOOLEAN NOT NULL DEFAULT 0
);


CREATE TABLE cycles (
    id INTEGER PRIMARY KEY,
    task_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(task_id) REFERENCES tasks(id)
);
