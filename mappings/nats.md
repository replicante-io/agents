## NATS Straming
NATS itself seems to be a network abstraction and not a datastore
(there seems to be no persistence) so it is not supported.


* Administration:
  * A cluster-unique name for the node: `server_id` from `/streaming/serverz`.
  * Cluster name shared by all nodes: user defined in agent configuration (only available in the API when node is `FT_ACTIVE`).
  * Version information: `version` from `/streaming/serverz`.

* Clustering: the NATS straming process, MUST BE in fault tolerant mode.

* Replication:
  * For each node, which shards are on the node: only one, the fault tolerance group.
  * For each shard on each node, what the role of the node is: user defined in agent configuration (can't get FT name from API).
  * For each non-primary shard on each node, the replication lag for the node: `null` (technically depends on backing store).

* Sharding:
  * What is a shard: a fault tolerance group.
  * What is a shard ID: the FT group name, user defined in agent configuration.
  * For each shard, the time of the last operation: UNAVAILBLE.
