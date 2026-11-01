#include "../src/telemetry.hpp"

#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>

namespace {

int failures = 0;

void expect(bool condition, const char* name) {
    if (!condition) {
        ++failures;
        std::cerr << "FAIL: " << name << '\n';
    }
}

std::string read_file(const std::filesystem::path& path) {
    std::ifstream input(path);
    return std::string((std::istreambuf_iterator<char>(input)), std::istreambuf_iterator<char>());
}

} // namespace

int main() {
    using uor_inverted_file::Attribute;
    using uor_inverted_file::Resource;
    using uor_inverted_file::Span;
    using uor_inverted_file::SpanIds;
    using uor_inverted_file::Status;
    using uor_inverted_file::Telemetry;
    const std::filesystem::path output = "build/telemetry-test.json";
    std::filesystem::remove(output);

    Telemetry disabled;
    const Resource resource{{{"uor.artifact.payload_sha256", "abc123"},
                             {"uor.artifact.identity_kind", "fixed-codebook"}}};
    const Span span{"uor.cuda.index.build", SpanIds{"0123456789abcdef0123456789abcdef", "0123456789abcdef", ""},
                    100, 200, {{"uor.operation.status", "ok"}}};
    expect(!disabled.enabled(), "default telemetry is disabled");
    expect(disabled.export_trace(resource, span) == Status::Ok, "disabled export is a no-op");
    expect(!std::filesystem::exists(output), "disabled export creates no file");

    Telemetry enabled(output);
    expect(enabled.enabled(), "explicit output enables telemetry");
    expect(enabled.export_trace(resource, span) == Status::Ok, "explicit export succeeds");
    const auto json = read_file(output);
    expect(json.find("\"resourceSpans\"") != std::string::npos, "resourceSpans present");
    expect(json.find("\"scopeSpans\"") != std::string::npos, "scopeSpans present");
    expect(json.find("\"spans\"") != std::string::npos, "spans present");
    expect(json.find("\"traceId\":\"0123456789abcdef0123456789abcdef\"") != std::string::npos,
           "trace ID serialized");
    expect(json.find("\"spanId\":\"0123456789abcdef\"") != std::string::npos, "span ID serialized");
    expect(json.find("\"startTimeUnixNano\":\"100\"") != std::string::npos, "start timestamp serialized");
    expect(json.find("\"endTimeUnixNano\":\"200\"") != std::string::npos, "end timestamp serialized");
    expect(json.find("\"uor.artifact.payload_sha256\"") != std::string::npos, "artifact digest attribute");
    expect(json.find("\"uor.artifact.identity_kind\"") != std::string::npos, "identity kind attribute");
    expect(json.find("prompt") == std::string::npos, "no prompt content");
    expect(json.find("\"code\":0") != std::string::npos, "success status serialized");

    Span child{"uor.cuda.index.probe",
               SpanIds{span.ids.trace_id, "fedcba9876543210", span.ids.span_id}, 200, 300, {}};
    expect(enabled.export_trace(resource, std::vector<Span>{span, child}) == Status::Ok,
           "batch export succeeds");
    const auto batch = read_file(output);
    expect(batch.find(span.name) != std::string::npos && batch.find(child.name) != std::string::npos,
           "one JSON document retains every operation");
    expect(batch.find("\"parentSpanId\":\"0123456789abcdef\"") != std::string::npos,
           "batch preserves the child parent link");

    Span invalid = span;
    invalid.ids.trace_id = "not-a-trace-id";
    expect(enabled.export_trace(resource, invalid) == Status::InvalidArgument, "invalid ID refused");

    if (failures != 0) {
        std::cerr << failures << " telemetry exporter assertions failed\n";
        return 1;
    }
    std::cout << "telemetry exporter tests: default-off, OTLP shape, attributes, and refusal checks passed\n";
    return 0;
}
