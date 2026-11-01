#include "telemetry.hpp"

#include <fstream>
#include <iomanip>
#include <sstream>
#include <string_view>

namespace uor_inverted_file {
namespace {

std::string escape(std::string_view input) {
    std::ostringstream output;
    for (const unsigned char c : input) {
        switch (c) {
            case '"': output << "\\\""; break;
            case '\\': output << "\\\\"; break;
            case '\b': output << "\\b"; break;
            case '\f': output << "\\f"; break;
            case '\n': output << "\\n"; break;
            case '\r': output << "\\r"; break;
            case '\t': output << "\\t"; break;
            default:
                if (c < 0x20) {
                    output << "\\u00" << std::hex << std::setw(2) << std::setfill('0')
                           << static_cast<unsigned>(c) << std::dec;
                } else {
                    output << c;
                }
        }
    }
    return output.str();
}

std::string attributes(const std::vector<Attribute>& values) {
    std::ostringstream output;
    output << '[';
    for (std::size_t i = 0; i < values.size(); ++i) {
        if (i != 0) output << ',';
        output << "{\"key\":\"" << escape(values[i].key) << "\",\"value\":{\"stringValue\":\""
               << escape(values[i].value) << "\"}}";
    }
    output << ']';
    return output.str();
}

bool valid_id(const std::string& id, std::size_t length) {
    if (id.size() != length) return false;
    if (id == std::string(length, '0')) return false;
    for (const char c : id) {
        if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'))) return false;
    }
    return true;
}

} // namespace

Telemetry::Telemetry(std::filesystem::path output) : output_(std::move(output)) {}

bool Telemetry::enabled() const { return !output_.empty(); }

Status Telemetry::export_trace(const Resource& resource, const Span& span) const {
    if (output_.empty()) return Status::Ok;
    if (span.name.empty() || !valid_id(span.ids.trace_id, 32) ||
        !valid_id(span.ids.span_id, 16) || span.end_unix_nano < span.start_unix_nano ||
        (!span.ids.parent_span_id.empty() && !valid_id(span.ids.parent_span_id, 16))) {
        return Status::InvalidArgument;
    }
    std::ostringstream json;
    json << "{\"resourceSpans\":[{\"resource\":{\"attributes\":"
         << attributes(resource.attributes)
         << "},\"scopeSpans\":[{\"scope\":{\"name\":\"uor-cuda-inverted-file\"},"
         << "\"spans\":[{\"traceId\":\"" << span.ids.trace_id
         << "\",\"spanId\":\"" << span.ids.span_id << "\",";
    if (!span.ids.parent_span_id.empty()) {
        json << "\"parentSpanId\":\"" << span.ids.parent_span_id << "\",";
    }
    json << "\"name\":\"" << escape(span.name)
         << "\",\"kind\":1,\"startTimeUnixNano\":\"" << span.start_unix_nano
         << "\",\"endTimeUnixNano\":\"" << span.end_unix_nano
         << "\",\"attributes\":" << attributes(span.attributes)
         << ",\"status\":{\"code\":0}}]}]}]}";
    std::ofstream output(output_, std::ios::trunc);
    if (!output) return Status::CudaFailure;
    output << json.str() << '\n';
    return output.good() ? Status::Ok : Status::CudaFailure;
}

} // namespace uor_inverted_file
