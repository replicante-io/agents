## Etcd
* Administration:
  * A cluster-unique name for the node: node name (extract node ID from `MemberListResponse.ResponseHeader.member_id` and map to a `MemberListResponse.Member`).
  * Cluster name shared by all nodes: user defined in agent configuration.
  * Version information: `StatusResponse.version`.

* Clustering: etcd processes.

* Replication:
  * For each node, which shards are on the node: the entire dataset.
  * For each shard on each node, what the role of the node is: `StatusResponse.leader`.
  * For each non-primary shard on each node, the replication lag for the node: `StatusResponse.raftIndex` (primary - node).

* Sharding:
  * What is a shard: the entire dataset.
  * What is a shard ID: the name of the cluster.
  * For each shard, the time of the last operation: the `StatusResponse.raftIndex` raft offset.
