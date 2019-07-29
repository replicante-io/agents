-- Based on ActionRecord from base/src/actions/definition.rs
CREATE TABLE IF NOT EXISTS actions(
  action TEXT NOT NULL,
  agent_version TEXT NOT NULL,
  args TEXT NOT NULL,
  created_ts TEXT NOT NULL,
  headers TEXT NOT NULL,
  id TEXT PRIMARY KEY NOT NULL,
  requester TEXT NOT NULL,
  state TEXT NOT NULL
);
