---
id: intro-security
title: Security Considerations
sidebar_label: Security Considerations
---

This page focuses on security considerations of running agents.

The security considerations for the overall Replicante ecosystem design are documented in the
security section of the [admin manual](https://www.replicante.io/docs/manual/docs/security/).


## HTTPS
By default communication with replicante core happens over HTTP, with core initiating connections.

This is not a secure setup (unless you trust the network, but you don't right?).
At the very least this exposes the system to [replay attacks](https://en.wikipedia.org/wiki/Replay_attack):
a malicious user can record a legitimate request and re-send it to the agent at will.

Offical agents support HTTPS-only servers,
with mutual certificte verifiction required for actions to be enabled.

<blockquote class="info">

The actions system can only be enabled if mutual HTTPS verification is enforced.
This is by design as actions can easily compromise datastores if used casually or maliciously.

</blockquote>


## Runtime user
The agent may require some privileges on a server to perform actions
such as restart datastores or update TLS certificates.

The recommended approach is to run the agent under a user with limited permissions
and grant extra permissions as and when required.

Specific agent's documentation will provide extra information and details
regarding the permissions needed by the agent itself.
