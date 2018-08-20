## CrateDB
* Administration:
  * A cluster-unique name for the node: user defined in agent configuration (MUST match local node name).
  * Cluster name shared by all nodes: [`SELECT name FROM sys.cluster;`](https://crate.io/docs/crate/reference/en/latest/admin/system-information.html#cluster).
  * Version information: [`SELECT name, version['number'] as version FROM sys.nodes WHERE name = '<NODE>';`](https://crate.io/docs/crate/reference/en/latest/admin/system-information.html#version).

* Clustering: CrateDB processes.

* Replication:
  * For each node, which shards are on the node: [`SELECT schema_name, table_name, partition_ident, id FROM sys.shards WHERE node['name'] = '<NODE>';`](https://crate.io/docs/crate/reference/en/latest/admin/system-information.html#shards)
  * For each shard on each node, what the role of the node is: [`SELECT schema_name, table_name, partition_ident, id, primary FROM sys.shards WHERE node['name'] = '<NODE>';`](https://crate.io/docs/crate/reference/en/latest/admin/system-information.html#shards)
  * For each non-primary shard on each node, the replication lag for the node: bytes still to recover `recovery['size']['used'] - recovery['size']['recovered']`.

* Sharding:
  * What is a shard: a CrateDB shard.
  * What is a shard ID: `SCHEMA_NAME/TABLE_NAME/PARTITION_IDENT/ID`.
  * For each shard, the time of the last operation: not available.
