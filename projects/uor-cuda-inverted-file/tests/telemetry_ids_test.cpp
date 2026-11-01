#include "../src/sha256.hpp"
#include "../src/telemetry_ids.hpp"

#include <cctype>
#include <iostream>
#include <string>
#include <unordered_set>

namespace {

int failures = 0;

void expect(bool condition, const char* name) {
    if (!condition) {
        ++failures;
        std::cerr << "FAIL: " << name << '\n';
    }
}

bool lowercase_hex(const std::string& value) {
    for (const char c : value) {
        if (!std::isxdigit(static_cast<unsigned char>(c)) || (c >= 'A' && c <= 'F')) {
            return false;
        }
    }
    return true;
}

} // namespace

int main() {
    using uor_inverted_file::new_span_id;
    using uor_inverted_file::new_trace_id;
    using uor_inverted_file::sha256_hex;
    using uor_inverted_file::SpanIds;

    expect(sha256_hex(nullptr, 0) == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
           "SHA-256 empty vector");
    expect(sha256_hex(reinterpret_cast<const std::uint8_t*>("abc"), 3) ==
               "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
           "SHA-256 abc vector");

    std::unordered_set<std::string> traces;
    std::unordered_set<std::string> spans;
    for (int i = 0; i < 1000; ++i) {
        const auto trace = new_trace_id();
        const auto span = new_span_id();
        expect(trace.size() == 32 && lowercase_hex(trace), "trace ID is 32 lowercase hex");
        expect(span.size() == 16 && lowercase_hex(span), "span ID is 16 lowercase hex");
        expect(trace != std::string(32, '0'), "trace ID is not all zero");
        expect(span != std::string(16, '0'), "span ID is not all zero");
        traces.insert(trace);
        spans.insert(span);
    }
    expect(traces.size() == 1000, "root trace IDs are unique");
    expect(spans.size() == 1000, "span IDs are unique within a trace");

    const auto trace = new_trace_id();
    const auto parent = new_span_id();
    const auto child = new_span_id();
    const SpanIds context{trace, child, parent};
    expect(context.trace_id == trace, "child shares trace ID");
    expect(context.parent_span_id == parent && parent.size() == 16,
           "child parent is a 16-hex span ID");
    expect(context.span_id != context.parent_span_id, "child has its own span ID");

    if (failures != 0) {
        std::cerr << failures << " identifier assertions failed\n";
        return 1;
    }
    std::cout << "telemetry ID tests: all hashing, width, uniqueness, and parent checks passed\n";
    return 0;
}
