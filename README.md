# Hasura GraphQL Engine metric adapter

[Hasura GraphQL Engine](https://hasura.io) is an awesome piece of software, check it out! There is also a great cloud offering and a pro version, which holds a lot more very useful features.

For small projects, where I wanted to self host the graphql engine, it is laking visibility in terms of metrics. This project can help with this, since it reads and parses the hasura logs and collects and provides prometheus metrics from it.

The logs can expected to be read from a log file.

The application will start a web server on `${LISTEN_ADDR}`, where the metrics
can be scraped from the `/metrics` path.

Hasura should be configured to at least emit `http-log`, `webhook-log` and `query-log` log types.

Other envvars needed for configuration:

`HASURA_GRAPHQL_ENDPOINT` The hasura endpoint (defaults to `http://localhost:8080`)
`HASURA_GRAPHQL_ADMIN_SECRET` The hasura admin secret this is required

## Metrics

- `hasura_log_lines_counter`
    This is a counter the counts all parsed log lines. The labels include the
    log type.

- `hasura_log_lines_counter_total`
    This is a counter that is the sum of all conted log lines.

- `hasura_query_execution_seconds`

    This is a histogram, that stores the query execution time in seconds.
    The labels are:
    - `operation` which holds the operation name of the graphql query or nothing
    if none is provided.
    - `error` which holds the error code if an error was detected or nothing if
    this was successfull

    For each label setting there are the following entries:
    - `hasura_query_execution_seconds_bucket`
    - `hasura_query_execution_seconds_sum`
    - `hasura_query_execution_seconds_count`

- `hasura_request_counter`

    This is a counter that cound the number of http requests. It provides
    `status` the http status code and `url` the path that was called.

- `hasura_request_query_counter`

    This is a counter that counts the number of queries.
    The labels are:
    - `operation` which holds the operation name of the graphql query or nothing
    if none is provided.
    - `error` which holds the error code if an error was detected or nothing if
    this was successfull

- `hasura_websockets_active`

    This is a gauge that holds the currently active websocket connections.

- `hasura_websockets_operations`

    This is a coutner that counts the operations (queries) over websockets.
    The labels are:
    - `operation` which holds the operation name of the graphql query or nothing
    if none is provided.
    - `error` which holds the error code if an error was detected or nothing if
    this was successfull


- `hasura_websockets_operations_active`

    This is a gauge that holds the currently active websocket operations.

- `hasura_healthy`

    This is a gauge that is 1 if the instance is healthy or 0 otherwise

The following metrics are the same as in the project (https://github.com/zolamk/hasura-exporter), also the idea on how to access them is based on it. So all credit for these need to go to @zolamk, I just ported them here.

- `hasura_pending_cron_triggers`, `hasura_processed_cron_triggers`, `hasura_successful_cron_triggers`, `hasura_failed_cron_triggers`

    These are gauges, that shows the number of (pending, processed, successful, failed) cron triggers labeled with the trigger name

- `hasura_pending_event_triggers`, `hasura_processed_event_triggers`, `hasura_successful_event_triggers`, `hasura_failed_event_triggers`

    These are gauges, that shows the number of (pending, processed, successful, failed) event triggers labeled with the trigger name

- `hasura_pending_one_off_events`, `hasura_processed_one_off_events`, `hasura_successful_one_off_events`, `hasura_failed_one_off_events`

    These are gauges, that shows the number of (pending, processed, successful, failed) one off events

- `hasura_metadata_consistency_status`

    This is a gauge that is 1 if the instance metadata is consistent or 0 otherwise

## Docker Image

Don't use version `v0.1.0` its broken.

The docker image `ghcr.io/afitzek/hasura-metric-adapter:v0.1.4` needs four environment variables to be configured.

`LISTEN_ADDR`: The listen address for the metric endpoint
`LOG_FILE`: The log file, which will hold the hasura logs
`HASURA_GRAPHQL_ENDPOINT` The hasura endpoint (defaults to `http://localhost:8080`)
`HASURA_GRAPHQL_ADMIN_SECRET` The hasura admin secret this is required

## K8S Example

An example k8s setup can be found in [k8s_example.yaml](k8s_example.yaml).

The important pieces of the example are:
- The adapter runs in sidecar mode with the hasura container
- The containers use a shared namespace, so that a named pipe can be accessed in both containers
    `shareProcessNamespace: true`
- The hasura container and the adapter share a common volume called `logs`, which is mounted in both containers as `/tmp/log`.
- Overwrite the hasura command to:
    ```
    "/bin/sh", "-c", "rm -rf /tmp/log/stdout.log && mkfifo /tmp/log/stdout.log && /bin/graphql-engine serve | tee /tmp/log/stdout.log"
    ```
    This creates a named pipe, and pipes the logs of graphql-engine to the stdout for logging and to the named pipe for the metric adapter to collect.

    (an alternative if you can't have shared process namespaces in the pod, is to use a file, but as @Hongbo-Miao pointed out in https://github.com/afitzek/hasura-metric-adapter/issues/11 the log file can become very big)
    ```
    "/bin/sh", "-c", ": > /tmp/log/stdout.log && /bin/graphql-engine serve | tee /tmp/log/stdout.log"
    ```
    This truncates the log file, to not count metrics on container restarts, starts the graphql-engine and pipes the stdout to stdout and the file `/tmp/log/stdout.log`.
- `HASURA_GRAPHQL_ENABLED_LOG_TYPES` includes `http-log`, `webhook-log` and `query-log`.
- The metric adapter is set up to listen on port `9999` and read the log from the shared volume `/tmp/log/stdout.log`.