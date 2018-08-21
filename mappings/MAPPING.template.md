## ???
* Administration:
  * A cluster name shared by all nodes: ???
  * A cluster-unique name for the node: ???
  * Version information: ???

* Clustering: ???

* Sharding: (A shard is ???)
  * A shard ID: ???
  * [Optional] An indicator of when the last write operation happened (commit offset):
    * A commit offset unit (i.e, seconds, commits, ...): ???
    * A commit offset value (as a 64-bits integer): ???

* Replication:
  * Which shards are on the node: ???
  * For each shard, what the role on the node is: ???
  * [Optional] For each non-primary shard, the replication lag:
    * The replication lag unit (i.e, seconds, commits, ...): ???
    * The replication lag value (as a 64-bits integer): ???
