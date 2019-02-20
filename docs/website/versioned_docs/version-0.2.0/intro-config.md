---
id: version-0.2.0-intro-config
title: Configuration
sidebar_label: Configuration
original_id: intro-config
---

As agents are delveloped as independent programs, configuration options
(and sometimes format) differ from agent to agent.

Official agents and those implemented using the base rust crate provide a set of standard
configuration options as well as agent specific options.
Below are the common agent options while agent specific options
are documented with the agents details.

```yaml
# Datastore independent agent configuration.
agent:
  # The section below is for the API interface configuration.
  api:
    # The network interface and port to bind the API server onto.
    #
    # By default, only bind to the loopback interface.
    # Production environments should place an HTTPS proxy in front of the API.
    bind: '127.0.0.1:8000'

  # The section below is for logging configuration.
  logging:
    # Flush logs asynchronously.
    # 
    # Pro:
    #     Async log flushing is more efficient as processes
    #     are not blocked waiting for logging backends to complete.
    # 
    # Con:
    #     If the process crashes logs in the buffer may be lost.
    #
    # Recommendation:
    #     Keep async logging enabled unless replicante is crashing
    #     and the logs don't have any indications of why.
    #
    #     Async logging may also be disabled in testing, debugging,
    #     or developing environments.
    async: true

    # Logging backend configuration.
    backend:
      # The backend to send logs to.
      # This option also determines the format and destination of logs.
      #
      # Available options:
      #
      #   * 'json': prints JSON formatted logs to standard output.
      #   * 'journald': sends logs to systemd journal (if enabled at compile time).
      name: json

      # Any backend-specific option is set here.
      # The available options vary from backend to backend and are documented below.
      #
      # *** None available at this time ***
      #options:

    # The minimum logging level.
    #
    # Available options:
    #
    #   * 'critical'
    #   * 'error'
    #   * 'warning'
    #   * 'info'
    #   * 'debug' (only available in debug builds)
    level: info


  # The section below is for distributed tracing configuration.
  tracing:
    # The distributed tracing backend to integrate with.
    #
    # Available options:
    #
    #   * 'noop'
    #   * 'zipkin'
    backend: noop

    # Any backend-specific option is set here.
    # The available options vary from tracer to tracer and are documented below.
    #
    # Zipkin options
    #options:
    #  # (required) The service name for this zipkin endpoint.
    #  service_name: replicante
    #
    #  # (required) List of kafka seed hostnames.
    #  kafka:
    #    - HOST1:9092
    #    - HOST2:9092
    #
    #  # The kafka topic to publish spans to.
    #  topic: zipkin
```