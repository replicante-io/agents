## Redis Cluster
* Administration:
  * A cluster-unique name for the node: Node ID from the [`CLUSTER NODES`](https://redis.io/commands/cluster-nodes) output.
  * Cluster name shared by all nodes: user defined in agent configuration.
  * Version information: `redis_version` field of the `server` section from the [`INFO`](https://redis.io/commands/info) output.

* Clustering: redis processes forming the cluster.

* Replication:
  * For each node, which shards are on the node: `self` node from the [`CLUSTER NODES`](https://redis.io/commands/cluster-nodes) output.
  * For each shard on each node, what the role of the node is: `self` node from the [`CLUSTER NODES`](https://redis.io/commands/cluster-nodes) output.
  * For each non-primary shard on each node, the replication lag for the node: replication offsets from the `replication` section from the [`INFO`](https://redis.io/commands/info) output.

* Sharding:
  * What is a shard: a set of hash slots allocated to the same node.
  * What is a shard ID: a bitmask of allocated slots in the shard.
  * For each shard, the time of the last operation: unavailable.
