# Hasura GraphQL Engine metric adapter

[Hasura GraphQL Engine](https://hasura.io) is an awesome piece of software, check it out! There is also a great cloud offering and a pro version, which holds a lot more very useful features.

For small projects, where I wanted to self host the graphql engine, it is laking visibility in terms of metrics. This project can help with this, since it reads and parses the hasura logs and collects and provides prometheus metrics from it.

The logs can expected to be read from a log file.

The application will start a web server on `${LISTEN_ADDR}`, where the metrics
can be scraped from the `/metrics` path.

Hasura should be configured to at least emit `http-log`, `webhook-log` and `query-log` log types.


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


## Docker Image

Don't use version `v0.1.0` its broken.

The docker image `ghcr.io/afitzek/hasura-metric-adapter:v0.1.1` needs two environment variables to be configured.

`LISTEN_ADDR`: The listen address for the metric endpoint
`LOG_FILE`: The log file, which will hold the hasura logs

## K8S Example

An example k8s setup can be found in [k8s_example.yaml](k8s_example.yaml).

The important pieces of the example are:
- The adapter runs in sidecar mode with the hasura container
- The hasura container and the adapter share a common volume called `logs`, which is mounted in both containers as `/tmp/log`.
- Overwrite the hasura command to:
    ```
    "/bin/sh", "-c", ": > /tmp/log/stdout.log && /bin/graphql-engine serve | tee /tmp/log/stdout.log"
    ```
    This truncates the log file, to not count metrics on container restarts, starts the graphql-engine and pipes the stdout to stdout and the file `/tmp/log/stdout.log`.
- `HASURA_GRAPHQL_ENABLED_LOG_TYPES` includes `http-log`, `webhook-log` and `query-log`.
- The metric adapter is set up to listen on port `9999` and read the log file from the shared volume `/tmp/log/stdout.log`.