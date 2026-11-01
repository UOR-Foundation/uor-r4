# Loopback OTLP collector check

This is an explicit interoperability experiment, not a service or a default telemetry mode. It uses the official OpenTelemetry Collector Contrib `0.164.0` image and the sibling `collector.yaml`, which listens only on `127.0.0.1:4318` and writes received spans to the collector's debug log.

From `projects/uor-cuda-inverted-file`:

```sh
make build/instrumented_example
UOR_OTEL_OUT=build/trace.json ./build/instrumented_example

docker run --rm -d --name uor-otel-collector \
  -v "$PWD/examples/otel/collector.yaml:/etc/otelcol-contrib/config.yaml:ro" \
  otel/opentelemetry-collector-contrib:0.164.0
docker run --rm -i --network container:uor-otel-collector \
  -v "$PWD/build:/tmp:ro" \
  curlimages/curl@sha256:3fb7e13e5d179a845c97d78204f3f1064d1907037f26a3ebf91710571151fd93 \
  --noproxy '*' -sS -D - -H 'Content-Type: application/json' \
  --data-binary @/tmp/trace.json -w '\nHTTP %{http_code}\n' \
  http://127.0.0.1:4318/v1/traces
docker logs uor-otel-collector
docker stop uor-otel-collector
```

The log must contain all four operation names, the resource's SHA-256 and 512-byte codebook metadata, and the emitted trace/span IDs. A successful POST is not enough on its own; inspect the decoded spans. Stop and remove the container after the check. No prompt/completion text, user-turn hashes, credentials, external endpoint, or production deployment is involved. The application still creates no telemetry file unless `UOR_OTEL_OUT` is explicitly set, and it contains no network exporter.
