## Redis (cluster)
* Administration:
  * A cluster ID shared by all nodes: user defined in agent configuration.
  * A cluster-unique ID for the node: Node ID from the [`CLUSTER NODES`](https://redis.io/commands/cluster-nodes) output.
  * Version information: `redis_version` field of the `server` section from the [`INFO`](https://redis.io/commands/info) output.
  * [Optional] An operation friendly cluster display name: unavailable.

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


## Redis (sentinel)
[Redis sentinel](https://redis.io/topics/sentinel) can be set up in many different ways.
Supporting all possible setups would not be an easy feat and would lead to far too much complexity.

Replicante support for redis with redis sentinel is limited to the following configuration:

  * Each redis process has a sentinel process.
  * The sentinel process runs on the same server as the redis process.
  * The sentinel is responsible for one master set only.

This is [example 2](https://redis.io/topics/sentinel#example-2-basic-setup-with-three-boxes)
from the redis sentinel documentation.


* Administration:
  * A cluster ID shared by all nodes: user defined in agent configuration.
  * A cluster-unique ID for the node: user defined in agent configuration.
  * Version information: `redis_version` field of the `server` section from the [`INFO`](https://redis.io/commands/info) output.
  * [Optional] An operation friendly cluster display name: unavailable.

* Clustering:
  * Redis processes holding the data.
  * Sentinel processes monitoring servers and performing failovers.

* Sharding: (A shard is the entire dataset)
  * A shard ID:
    * Redis: the name of the cluster.
    * Sentinel: none.
  * [Optional] An indicator of when the last write operation happened (commit offset):
    * Redis:
      * A commit offset unit (i.e, seconds, commits, ...): offset.
      * A commit offset value (as a 64-bits integer): `master_repl_offset` or `slave_repl_offset` from the `replication` section from the [`INFO`](https://redis.io/commands/info) output.
    * Sentinel: none.

* Replication:
  * Which shards are on the node:
    * Redis: the entire dataset.
    * Sentinel: none.
  * For each shard, what the role on the node is:
    * Redis: `role` from the `replication` section from the [`INFO`](https://redis.io/commands/info) output.
    * Sentinel: none.
  * [Optional] For each non-primary shard, the replication lag:
    * Redis:
      * The replication lag unit (i.e, seconds, commits, ...): offset.
      * The replication lag value (as a 64-bits integer): `master_repl_offset - slave_repl_offset` from the `replication` section from the [`INFO`](https://redis.io/commands/info) output.
    * Sentinel: none.
