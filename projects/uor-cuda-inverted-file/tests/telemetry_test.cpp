#include "../src/telemetry.hpp"
#include "json.hpp"

#include <array>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>
#include <thread>
#include <vector>

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
    expect(test_json::valid(json), "export is one syntactically valid JSON document");
    expect(!test_json::valid("{\"resourceSpans\":[]}\n{\"resourceSpans\":[]}"),
           "JSON parser rejects multiple documents");
    expect(!test_json::valid("{\"resourceSpans\":[]"), "JSON parser rejects a missing closing brace");
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

    expect(test_json::valid(batch), "batch is one valid JSON document");
    const Resource escaped{{{"quote\" slash\\ line\n tab\t snowman \xE2\x98\x83",
                             "control\x01 and \xE2\x98\x83"}}};
    expect(enabled.export_trace(escaped, span) == Status::Ok, "escaped Unicode export succeeds");
    const auto escaped_json = read_file(output);
    expect(test_json::valid(escaped_json), "escaped output remains valid JSON and UTF-8");
    expect(escaped_json.find("quote\\\" slash\\\\ line\\n tab\\t") != std::string::npos,
           "quotes, slashes, and control characters are escaped");
    expect(escaped_json.find("control\\u0001") != std::string::npos, "control byte uses a Unicode escape");

    const Resource malformed{{{"bad.utf8", std::string("\xC3\x28", 2)}}};
    const auto before_malformed = read_file(output);
    expect(enabled.export_trace(malformed, span) == Status::InvalidArgument, "malformed UTF-8 refused");
    expect(read_file(output) == before_malformed, "refusal preserves the last complete document");

    Span invalid = span;
    invalid.ids.trace_id = "not-a-trace-id";
    expect(enabled.export_trace(resource, invalid) == Status::InvalidArgument, "invalid ID refused");

    const std::filesystem::path failure_root = "build/telemetry-failure";
    std::filesystem::remove_all(failure_root);
    std::filesystem::create_directory(failure_root);
    Telemetry directory_output(failure_root);
    expect(directory_output.export_trace(resource, span) == Status::TelemetryFailure,
           "file replacement failure has a telemetry-specific status");
    expect(std::filesystem::is_directory(failure_root), "failed replacement preserves the destination");
    bool temporary_leftover = false;
    const auto temporary_prefix = failure_root.filename().string() + ".tmp.";
    for (const auto& entry : std::filesystem::directory_iterator(failure_root.parent_path())) {
        const auto name = entry.path().filename().string();
        if (name.compare(0, temporary_prefix.size(), temporary_prefix) == 0) temporary_leftover = true;
    }
    expect(!temporary_leftover, "failed replacement removes every exporter-owned temporary artifact");
    std::filesystem::remove_all(failure_root);

    const std::filesystem::path unrelated = "build/telemetry-test.json.tmp";
    {
        std::ofstream file(unrelated, std::ios::trunc);
        file << "preserve unrelated bytes";
    }
    expect(enabled.export_trace(resource, span) == Status::Ok, "export with a neighboring file succeeds");
    expect(read_file(unrelated) == "preserve unrelated bytes",
           "export never truncates or renames an unrelated temporary-looking file");
    std::filesystem::remove(unrelated);

    std::array<Status, 8> concurrent_status{};
    std::vector<std::thread> writers;
    for (std::size_t i = 0; i < concurrent_status.size(); ++i) {
        writers.emplace_back([&, i] {
            Telemetry concurrent(output);
            concurrent_status[i] = concurrent.export_trace(resource, span);
        });
    }
    for (auto& writer : writers) writer.join();
    for (const auto status : concurrent_status) {
        expect(status == Status::Ok, "concurrent writers use independent temporary files");
    }
    expect(test_json::valid(read_file(output)), "concurrent replacement leaves one complete JSON document");

    if (failures != 0) {
        std::cerr << failures << " telemetry exporter assertions failed\n";
        return 1;
    }
    std::cout << "telemetry exporter tests: default-off, OTLP shape, attributes, and refusal checks passed\n";
    return 0;
}
