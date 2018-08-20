## ElasticSearch
* Administration:
  * A cluster-unique name for the node: node's `name` as reported by [`_nodes/_local`](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/cluster-nodes-info.html).
  * Cluster name shared by all nodes: `cluster_name` as reported by [`_cluster/health?local=true`](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/cluster-health.html).
  * Version information: node's `version` as reported by [`_nodes/_local`](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/cluster-nodes-info.html).

* Clustering: elasticsearch instances forming the cluster.

* Replication:
  * For each node, which shards are on the node: [`_cat/shards?format=json&h=index,shard,prirep,node`](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/cat-shards.html).
  * For each shard on each node, what the role of the node is: [`_cat/shards?format=json&h=index,shard,prirep,node`](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/cat-shards.html).
  * For each non-primary shard on each node, the replication lag for the node: unavailable.

* Sharding:
  * What is a shard: an index's shard.
  * What is a shard ID: `INDEX/SHARD`.
  * For each shard, the time of the last operation: unavailable.
