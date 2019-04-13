## NATS Straming
NATS itself seems to be a network abstraction and not a datastore
(there seems to be no persistence) so it is not supported.


* Administration:
  * A cluster ID shared by all nodes: user defined in agent configuration (only available in the API when node is `FT_ACTIVE`).
  * A cluster-unique ID for the node: `server_id` from `/streaming/serverz`.
  * Version information: `version` from `/streaming/serverz`.
  * [Optional] An operation friendly cluster display name: unavailable.

* Clustering: the NATS straming process, MUST BE in fault tolerant mode.

* Sharding: (A shard is a fault tolerance group)
  * A shard ID: the FT group name, user defined in agent configuration.
  * [Optional] An indicator of when the last write operation happened (commit offset): unavailable.

* Replication:
  * Which shards are on the node: only one, the fault tolerance group.
  * For each shard, what the role on the node is: user defined in agent configuration (can't get FT name from API).
  * [Optional] For each non-primary shard, the replication lag: `null` (technically depends on backing store).
