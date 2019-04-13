## ElasticSearch
* Administration:
  * A cluster ID shared by all nodes: `cluster_name` as reported by [`_cluster/health?local=true`](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/cluster-health.html).
  * A cluster-unique ID for the node: node's `name` as reported by [`_nodes/_local`](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/cluster-nodes-info.html).
  * Version information: node's `version` as reported by [`_nodes/_local`](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/cluster-nodes-info.html).
  * [Optional] An operation friendly cluster display name: unavailable.

* Clustering: elasticsearch instances forming the cluster.

* Sharding: (A shard is an index's shard)
  * A shard ID: `INDEX/SHARD`.
  * [Optional] An indicator of when the last write operation happened (commit offset): unavailable.

* Replication:
  * Which shards are on the node: [`_cat/shards?format=json&h=index,shard,prirep,node`](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/cat-shards.html).
  * For each shard, what the role on the node is: [`_cat/shards?format=json&h=index,shard,prirep,node`](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/cat-shards.html).
  * [Optional] For each non-primary shard, the replication lag: unavailable.
