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
  * A cluster-unique name for the node: user defined in agent configuration.
  * Cluster name shared by all nodes: user defined in agent configuration.
  * Version information: output of `SELECT version();`

* Clustering: postgres server processes.
  Different clustering tools (like stolon) may need to be monitored in the future.

* Replication:
  * For each node, which shards are on the node: one (the database).
  * For each shard on each node, what the role of the node is: `SELECT pg_last_wal_receive_lsn() == NULL` on primary.
  * For each non-primary shard on each node, the replication lag for the node: `SELECT pg_current_wal_lsn() - SELECT pg_last_wal_receive_lsn()` (need access to the primary for this).

* Sharding:
  * What is a shard: the entire database.
  * What is a shard ID: the cluster name.
  * For each shard, the time of the last operation: unavailable.
