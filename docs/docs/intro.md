---
id: intro
title: Introduction
sidebar_label: Introduction
---

Agents are the part of the replicante ecosystem that ensure datastores conform with the model.

Beside the task of exporting datastores information in the correct format,
agents also implement administrate tasks that datastores do not implement directly.


## Existing Agents
The team behind replicante core provides official support for some agents.

The community is encouraged to develop many more agents (see below).
If you know of a community developed agent not listed below please open an
[issue](https://github.com/replicante-io/agents/issues/new) so it can be included.

Below is a list of knows agents:

  * [Kafka](agents-kafka.md) (official)
  * [MongoDB](agents-mongodb.md) (official)
  * [Zookeeper](agents-zookeeper.md) (official)


## Developing community agents
Official agents are built on top of a shared
[base agent rust crate](https://github.com/replicante-io/agents/tree/master/base).
This reduces code duplication and improves consistency across the ecosystem.

Using a shared base crate also mean that:

  * Agents don't have to re-implement common functionality that can be provided out of the box.
  * Implementation of the communication layer with the core system is taken care of.
  * Operational logic (logging, metrics, tracing, ...) is provided.
  * Many more features and tools ...

If you are looking to build a new agent in [rust](https://www.rust-lang.org/) take advantage of the
[`replicante_agent`](https://docs.rs/replicante_agent)
base crate to speed up the development and help us improve it.
