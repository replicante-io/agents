## Redis Cluster
* Administration:
  * A cluster name shared by all nodes: user defined in agent configuration.
  * A cluster-unique name for the node: Node ID from the [`CLUSTER NODES`](https://redis.io/commands/cluster-nodes) output.
  * Version information: `redis_version` field of the `server` section from the [`INFO`](https://redis.io/commands/info) output.

* Clustering: redis processes forming the cluster.

* Sharding: (A shard is a set of hash slots allocated to the same node)
  * A shard ID: a bitmask of allocated slots in the shard.
  * [Optional] An indicator of when the last write operation happened (commit offset):
    * A commit offset unit (i.e, seconds, commits, ...): offset.
    * A commit offset value (as a 64-bits integer): replication offset from the `replication` section from the [`INFO`](https://redis.io/commands/info) output.

* Replication:
  * Which shards are on the node: `self` node from the [`CLUSTER NODES`](https://redis.io/commands/cluster-nodes) output.
  * For each shard, what the role on the node is: `self` node from the [`CLUSTER NODES`](https://redis.io/commands/cluster-nodes) output.
  * [Optional] For each non-primary shard, the replication lag:
    * The replication lag unit (i.e, seconds, commits, ...): offsets
    * The replication lag value (as a 64-bits integer): replication offsets from the `replication` section from the [`INFO`](https://redis.io/commands/info) output.
