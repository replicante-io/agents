## PostgreSQL (stolon)
PostgreSQL 9.0+ introduces streaming replication and a few other features that
can be used to build automated failover and recovery.
While great work has been done, on its own PostgreSQL is not easy to use as
a replicated cluster (the upside is that many configurations can be efficiently
supported by PostgreSQL).

Thankfully projects already exist to wrap PostgreSQL native functionality.
One such project, [stolon](https://github.com/sorintlab/stolon), can be used
to manage a replicated PostgreSQL cluster.


* Administration:
  * A cluster name shared by all nodes: user defined in agent configuration.
  * A cluster-unique name for the node: user defined in agent configuration.
  * Version information: output of `SELECT version();`

* Clustering:
  * PostgreSQL server processes.
  * Different clustering tools (like stolon) may need to be monitored in the future.

* Sharding: (A shard is the entire database)
  * A shard ID: the cluster name.
  * [Optional] An indicator of when the last write operation happened (commit offset):
    * A commit offset unit (i.e, seconds, commits, ...): offset.
    * A commit offset value (as a 64-bits integer): `SELECT pg_last_wal_receive_lsn();`.

* Replication:
  * Which shards are on the node: one (the database).
  * For each shard, what the role on the node is: `SELECT pg_last_wal_receive_lsn() == NULL` on primary.
  * [Optional] For each non-primary shard, the replication lag: unavailable (need access to primary as well as local node).
