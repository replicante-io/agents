## Consul
* Administration:
  * A cluster name shared by all nodes: user defined in agent configuration.
  * A cluster-unique name for the node: `.Member.Name` from [`/v1/agent/self`](https://www.consul.io/api/agent.html#read-configuration).
  * Version information: `.Config.Version` from [`/v1/agent/self`](https://www.consul.io/api/agent.html#read-configuration).

* Clustering: consul server agent.

* Sharding: (A shard is the entire dataset)
  * A shard ID: the name of the cluster.
  * [Optional] An indicator of when the last write operation happened (commit offset):
    * A commit offset unit (i.e, seconds, commits, ...): offset.
    * A commit offset value (as a 64-bits integer): the `.Stats.raft.last_log_index` raft offest.

* Replication:
  * Which shards are on the node: the entire dataset.
  * For each shard, what the role on the node is: `.Stats.raft.state` from [`/v1/agent/self`](https://www.consul.io/api/agent.html#read-configuration).
  * [Optional] For each non-primary shard, the replication lag: unavailable (need access to primary as well as local node).
