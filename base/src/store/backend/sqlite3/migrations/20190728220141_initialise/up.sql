-- Based on ActionRecord from base/src/actions/definition.rs
CREATE TABLE IF NOT EXISTS actions(
  id TEXT PRIMARY KEY NOT NULL,
  action TEXT NOT NULL,
  agent_version TEXT NOT NULL,
  args TEXT NOT NULL,
  created_ts INTEGER NOT NULL,
  headers TEXT NOT NULL,
  requester TEXT NOT NULL,
  state TEXT NOT NULL,
  state_payload TEXT,
  -- Additional attributes NOT exposed in the model.
  finished_ts INTEGER DEFAULT NULL
);
CREATE INDEX actions_created_ts ON actions(created_ts);
CREATE INDEX actions_finished_ts ON actions(finished_ts);
CREATE INDEX actions_state ON actions(state);

-- Based on ActionRecordHistory from base/src/actions/definition.rs
CREATE TABLE IF NOT EXISTS actions_history(
  -- INTEGER PRIMARY KEY is an alias for ROWID (which is more efficient then AUTOINCREMENT).
  -- https://www.sqlite.org/autoinc.html
  id INTEGER PRIMARY KEY NOT NULL,
  action_id TEXT NOT NULL,
  time INTEGER NOT NULL,
  state TEXT NOT NULL,
  state_payload TEXT,
  FOREIGN KEY(action_id) REFERENCES actions(id) ON UPDATE RESTRICT ON DELETE CASCADE
);
