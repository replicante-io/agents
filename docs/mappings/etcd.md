## Etcd
* Administration:
  * A cluster ID shared by all nodes: user defined in agent configuration.
  * A cluster-unique ID for the node: node name (extract node ID from `MemberListResponse.ResponseHeader.member_id` and map to a `MemberListResponse.Member`).
  * Version information: `StatusResponse.version`.
  * [Optional] An operation friendly cluster display name: unavailable.

* Clustering: etcd processes.

* Sharding: (A shard is the entire dataset)
  * A shard ID: the name of the cluster.
  * [Optional] An indicator of when the last write operation happened (commit offset):
    * A commit offset unit (i.e, seconds, commits, ...): offset
    * A commit offset value (as a 64-bits integer): the `StatusResponse.raftIndex` raft offset.

* Replication:
  * Which shards are on the node: the entire dataset.
  * For each shard, what the role on the node is: `StatusResponse.leader`.
  * [Optional] For each non-primary shard, the replication lag: unavailable (need access to primary as well as local node).
