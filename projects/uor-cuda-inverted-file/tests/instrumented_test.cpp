#include "../src/instrumented.hpp"
#include "../src/sha256.hpp"
#include "json.hpp"

#include <chrono>
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

uor_inverted_file::Table fixture() {
    using uor_inverted_file::Block;
    using uor_inverted_file::Table;
    using uor_inverted_file::codebook;
    using uor_inverted_file::Token;
    Table table;
    for (std::uint32_t i = 0; i < 4; ++i) {
        const auto& first = codebook[i];
        const auto& last = codebook[(i * 5 + 1) % codebook.size()];
        table.tokens.push_back(Token{first, Block{}, Block{}, last});
    }
    return table;
}

} // namespace

int main() {
    using uor_inverted_file::Block;
    using uor_inverted_file::Index;
    using uor_inverted_file::Instrumented;
    using uor_inverted_file::Query;
    using uor_inverted_file::ScanResult;
    using uor_inverted_file::Status;
    using uor_inverted_file::Table;
    using uor_inverted_file::codebook;
    const std::filesystem::path disabled_output = "build/instrumented-disabled.json";
    const std::filesystem::path enabled_output = "build/instrumented-enabled.json";
    std::filesystem::remove(disabled_output);
    std::filesystem::remove(enabled_output);

    Instrumented disabled;
    Index disabled_index;
    expect(!disabled.enabled(), "instrumented default is disabled");
    expect(disabled.build(fixture(), disabled_index) == Status::Ok, "disabled build preserves behavior");
    expect(!std::filesystem::exists(disabled_output), "disabled instrumentation creates no file");

    Instrumented enabled(enabled_output);
    expect(enabled.enabled(), "explicit output enables instrumentation");
    Table table = fixture();
    Index index;
    expect(enabled.build(table, index) == Status::Ok, "instrumented build preserves behavior");
    table.tokens[1][3] = codebook[2];
    expect(enabled.update(table, 1, index) == Status::Ok, "instrumented update preserves behavior");
    Index expected;
    expect(uor_inverted_file::build(table, expected) == Status::Ok, "fresh comparison build succeeds");
    expect(index.offsets == expected.offsets && index.members == expected.members &&
               index.cells == expected.cells && index.absent_count == expected.absent_count,
           "instrumented update matches a fresh build");

    const Query query{codebook[0], Block{}, Block{}, codebook[1]};
    ScanResult bucket, full;
    expect(enabled.probe(table, query, 2, index, bucket) == Status::Ok, "instrumented probe preserves behavior");
    expect(enabled.full_scan(table, query, 2, full) == Status::Ok, "instrumented full scan preserves behavior");
    expect(bucket.prediction == full.prediction && bucket.admitted == full.admitted &&
               bucket.top8 == full.top8,
           "instrumented parity preserved");

    const auto json = read_file(enabled_output);
    expect(test_json::valid(json), "instrumented output is one valid JSON document");
    expect(json.find("\"key\":\"uor.artifact.byte_size\",\"value\":{\"stringValue\":\"512\"}") !=
               std::string::npos,
           "artifact size covers every hashed codebook byte");
    expect(json.find("uor.cuda.index.build") != std::string::npos, "build span emitted");
    expect(json.find("uor.cuda.index.update") != std::string::npos, "update span emitted");
    expect(json.find("uor.cuda.index.probe") != std::string::npos, "probe span emitted");
    expect(json.find("uor.cuda.index.full_scan") != std::string::npos, "full-scan span emitted");
    expect(json.find("uor.artifact.payload_sha256") != std::string::npos, "artifact digest emitted");
    expect(json.find("uor.artifact.identity_kind") != std::string::npos, "identity kind emitted");
    expect(json.find("uor.artifact.digest.algorithm") != std::string::npos, "digest algorithm emitted");
    expect(json.find("prompt") == std::string::npos && json.find("completion") == std::string::npos &&
               json.find("user_turn") == std::string::npos,
           "no user content or user-turn identity emitted");

    const auto field = [](const std::string& document, const std::string& name, std::size_t occurrence) {
        const std::string needle = "\"" + name + "\":\"";
        std::size_t position = 0;
        for (std::size_t i = 0; i <= occurrence; ++i) {
            position = document.find(needle, position);
            if (position == std::string::npos) return std::string{};
            position += needle.size();
        }
        const auto end = document.find('"', position);
        return end == std::string::npos ? std::string{} : document.substr(position, end - position);
    };
    const auto build_span = field(json, "spanId", 0);
    expect(build_span.size() == 16, "build parent ID emitted");
    for (std::size_t i = 0; i < 4; ++i) {
        expect(field(json, "traceId", i) == field(json, "traceId", 0), "operations share one trace");
        if (i != 0) expect(field(json, "parentSpanId", i - 1) == build_span, "child references build parent");
        expect(field(json, "startTimeUnixNano", i).size() >= 19 &&
                   field(json, "endTimeUnixNano", i).size() >= 19,
               "timestamps use Unix nanoseconds rather than microseconds");
    }

    const auto digest = uor_inverted_file::sha256_hex(
        reinterpret_cast<const std::uint8_t*>(codebook.data()), codebook.size() * sizeof(Block));
    expect(json.find(digest) != std::string::npos, "digest is stable and correct");

    Instrumented second("build/instrumented-second.json");
    Index second_index;
    expect(second.build(fixture(), second_index) == Status::Ok, "second build succeeds");
    const auto second_json = read_file("build/instrumented-second.json");
    expect(field(json, "traceId", 0) != field(second_json, "traceId", 0),
           "separate executions produce different trace IDs");
    expect(second_json.find(digest) != std::string::npos, "second execution keeps artifact digest");

    const std::filesystem::path missing_parent = "build/missing-parent";
    std::filesystem::remove_all(missing_parent);
    Instrumented failed_output(missing_parent / "trace.json");
    Index completed_index;
    expect(failed_output.build(fixture(), completed_index) == Status::TelemetryFailure,
           "telemetry I/O failure is not mislabeled as CUDA failure");
    expect(uor_inverted_file::validate(fixture(), completed_index) == Status::Ok,
           "telemetry failure does not undo or invalidate the completed CPU build");
    expect(failed_output.full_scan(fixture(), query, 0, full) == Status::InvalidArgument,
           "CPU operation errors take precedence over export errors");
    expect(uor_inverted_file::telemetry_timestamp_after(1000, std::chrono::nanoseconds(5)) == 1005,
           "monotonic elapsed time advances the wall-clock start");
    expect(uor_inverted_file::telemetry_timestamp_after(1000, std::chrono::nanoseconds(-5)) == 1000,
           "timestamp calculation cannot move the endpoint backwards");

    Instrumented repeated("build/instrumented-repeated.json");
    Index repeated_index;
    expect(repeated.build(fixture(), repeated_index) == Status::Ok, "first repeated-session build succeeds");
    expect(repeated.build(fixture(), repeated_index) == Status::Ok, "session remains usable on later calls");
    expect(test_json::valid(read_file("build/instrumented-repeated.json")),
           "repeated calls retain one valid document");

    if (failures != 0) {
        std::cerr << failures << " instrumented telemetry assertions failed\n";
        return 1;
    }
    std::cout << "instrumented telemetry tests: default-off, operation spans, artifact identity, and parity passed\n";
    return 0;
}
