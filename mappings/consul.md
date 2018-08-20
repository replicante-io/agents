## Consul
* Administration:
  * A cluster-unique name for the node: `.Member.Name` from [`/v1/agent/self`](https://www.consul.io/api/agent.html#read-configuration).
  * Cluster name shared by all nodes: user defined in agent configuration.
  * Version information: `.Config.Version` from [`/v1/agent/self`](https://www.consul.io/api/agent.html#read-configuration).

* Clustering: consul server agent.

* Replication:
  * For each node, which shards are on the node: the entire dataset.
  * For each shard on each node, what the role of the node is: `.Stats.raft.state` from [`/v1/agent/self`](https://www.consul.io/api/agent.html#read-configuration).
  * For each non-primary shard on each node, the replication lag for the node: `Stats.raft.last_log_index` (primary - node).

* Sharding:
  * What is a shard: the entire dataset.
  * What is a shard ID: the name of the cluster.
  * For each shard, the time of the last operation: the `.Stats.raft.last_log_index` raft offest.
