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
);
CREATE INDEX actions_created_ts ON actions(created_ts);
CREATE INDEX actions_state ON actions(state);
