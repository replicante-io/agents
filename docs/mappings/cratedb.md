## CrateDB
* Administration:
  * A cluster ID shared by all nodes: [`SELECT name FROM sys.cluster;`](https://crate.io/docs/crate/reference/en/latest/admin/system-information.html#cluster).
  * A cluster-unique ID for the node: user defined in agent configuration (MUST match local node name).
  * Version information: [`SELECT name, version['number'] as version FROM sys.nodes WHERE name = '<NODE>';`](https://crate.io/docs/crate/reference/en/latest/admin/system-information.html#version).
  * [Optional] An operation friendly cluster display name: unavailable.

* Clustering: CrateDB processes.

* Sharding: (A shard is a CrateDB table shard)
  * A shard ID: `SCHEMA_NAME/TABLE_NAME/PARTITION_IDENT/ID`.
  * [Optional] An indicator of when the last write operation happened (commit offset): unavailable.

* Replication:
  * Which shards are on the node: [`SELECT schema_name, table_name, partition_ident, id FROM sys.shards WHERE node['name'] = '<NODE>';`](https://crate.io/docs/crate/reference/en/latest/admin/system-information.html#shards)
  * For each shard, what the role on the node is: [`SELECT schema_name, table_name, partition_ident, id, primary FROM sys.shards WHERE node['name'] = '<NODE>';`](https://crate.io/docs/crate/reference/en/latest/admin/system-information.html#shards)
  * [Optional] For each non-primary shard, the replication lag:
    * The replication lag unit (i.e, seconds, commits, ...): bytes.
    * The replication lag value (as a 64-bits integer): bytes still to recover `recovery['size']['used'] - recovery['size']['recovered']`.
