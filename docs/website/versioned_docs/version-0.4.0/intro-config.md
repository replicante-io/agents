---
id: version-0.4.0-intro-config
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

    # The number of request handling threads.
    #
    # By default this is 8 * number of CPUs.
    threads_count: ~

    # API server timeouts.
    timeouts:
      # Controls the timeout, in seconds, for keep alive connections.
      #
      # NOTE: Setting this to null (~) will turn off keep alive.
      keep_alive: 5

      # Control the timeout, in seconds, for reads on existing connections.
      #
      # NOTE: Setting this to null (~) will turn off read timeouts.
      read: 5

      # Control the timeout, in seconds, for writes on existing connections.
      #
      # NOTE: Setting this to null (~) will turn off write timeouts.
      write: 1

    # Enable/disable entire API trees.
    #
    # Useful for advanced operators that which to control access to experimental or legacy
    # API versions or reduce attack surfices by removing endpoints that are not needed.
    #
    # Example use cases are:
    #
    #   * Upgrade prep: testing new API versions while having a quick rollback plan.
    #   * Controlled rollout: be prepared for when verions are no longer supported.
    #   * Disable unstable/experimental APIs: to enusre integrated tools only use stable APIs.
    trees:
      # Enable/disable the introspection APIs.
      #
      # The introspection API is very usesul to gain insight into the system.
      # It can also be used to monitor the system for failures or performance degradation.
      introspect: true

      # Enable/disable the unstable APIs.
      #
      # The unstable APIs are for endpoints in the early development cycle
      # where the attributes and parameters can change a lot and often.
      unstable: true

  # Override the cluster display name, or set it if none was detected.
  #
  # The cluster ID is used to uniquely identify the cluster across the system
  # but some datastores report random, machine generated, strings as IDs.
  # To make clusters more identifiable to Replicante users a display name is
  # used in messages/views when provided, using the cluster ID as a fallback.
  #
  # When a datastore does not auto-detect a display name or you wish to change the
  # auto-detected display name, use this field to set whatever you want.
  #
  # NOTE: just like the cluster ID, the display name must be unique across all
  # clusters in a single Replicante Core instance.
  cluster_display_name_override: ~

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

    # Advanced level configuration by module prefix.
    #
    # The keys in this map are used as prefix matches against log event modules.
    # If a match is found the mapped level is used for the event.
    # If no match is found the `level` value is used as the filter.
    #
    # Example:
    #
    #     modules:
    #       'hyper::server': debug
    #       'rdkafka': error
    #
    # To find out what modules are available you can set `level` to DEBUG
    # and enable `verbose` logging to see all logs.
    # Once you know what logs you are looking for you can undo the changes to `level` and `verbose`
    # and add the module prefix you need to the `modules` option.
    modules: {}

    # Enable verbose debug logs.
    #
    # When DEBUG level is enabled, things can get loud pretty easily.
    # To allow DEBUG level to be more useful, only application events are emitted at
    # DEBUG level while dependency events are emitted at INFO level.
    #
    # Verbose mode can be used in cases where DEBUG level should be enabled by default
    # on all events and not just the application logs.
    verbose: false


  # Optional sentry.io integration configuration (desabled by default).
  #
  # Set a DSN parameter to enable centralised error reporting.
  #sentry:
  #  # Sentry API response capture filter.
  #  #
  #  # When sentry is enabled, API error responses returned to clients can be recorded
  #  # as sentry events to detect and debug issues.
  #  #
  #  # This option sets the severity level of responses that are reported:
  #  #  * `no`: disable sentry capturing of API error responses.
  #  #  * `client`: report client side errors and above (status code >= 400).
  #  #  * `server`: only report server side errors (status code >= 500).
  #  capture_api_errors: 'server'
  #
  #  # (required) The DSN to use to configure sentry.
  #  dsn: 'https://key@server.domain:port/project'
  sentry: ~


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
    #  # (required) The transport to send tracing information to zipkin.
    #  #
    #  # Available options:
    #  #
    #  #  * 'http'
    #  #  * 'kafka'
    #  transport: 'http'
    #
    #  # Any transport-specific option is set here.
    #  # The available options vary and are documented below.
    #  #
    #  # HTTP transport options
    #  options:
    #    # Number of buffered spans that should trigger a flush.
    #    #
    #    # This option is a best-effort configuration and the size of the buffer may grow
    #    # slightly above this threshold.
    #    flush_count: 100
    #
    #    # Muximum delay between span flushes in milliseconds.
    #    #
    #    # This option is a best-effort configuration and the size of the buffer may grow
    #    # slightly above this threshold.
    #    flush_timeout_millis: 2000
    #
    #    # Custom headers to attach to POST requests.
    #    headers: {}
    #
    #    # (required) Target URL to POST spans to.
    #    url: 'https://zipkin.corp/'
    #
    #  # Kafka transport options
    #  options:
    #    # (required) List of kafka seed hostnames.
    #    kafka:
    #      - HOST1:9092
    #      - HOST2:9092
    #
    #    # The kafka topic to publish spans to.
    #    topic: zipkin

  # Enable the update checker (optional).
  #
  # The check is performed only once in the background as the process starts.
  # If a new version is available a notice will be logged and captured as a sentry event.
  #
  # This feature is disabled by default to ensure the user privacy is respected
  # (HTTP requests can be tracked).
  # If this feature is not enabled, you will have to make sure you keep replicante up to date.
  update_checker: false
```
