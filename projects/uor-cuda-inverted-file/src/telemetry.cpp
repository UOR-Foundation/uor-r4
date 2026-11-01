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

bool valid_utf8(std::string_view input) {
    std::size_t position = 0;
    while (position < input.size()) {
        const auto first = static_cast<unsigned char>(input[position]);
        if (first < 0x80) {
            ++position;
            continue;
        }
        std::size_t length = 0;
        std::uint32_t codepoint = 0;
        std::uint32_t minimum = 0;
        if (first >= 0xc2 && first <= 0xdf) {
            length = 2;
            codepoint = first & 0x1f;
            minimum = 0x80;
        } else if (first >= 0xe0 && first <= 0xef) {
            length = 3;
            codepoint = first & 0x0f;
            minimum = 0x800;
        } else if (first >= 0xf0 && first <= 0xf4) {
            length = 4;
            codepoint = first & 0x07;
            minimum = 0x10000;
        } else {
            return false;
        }
        if (position + length > input.size()) return false;
        ++position;
        for (std::size_t i = 1; i < length; ++i) {
            const auto next = static_cast<unsigned char>(input[position++]);
            if ((next & 0xc0) != 0x80) return false;
            codepoint = (codepoint << 6) | (next & 0x3f);
        }
        if (codepoint < minimum || codepoint > 0x10ffff ||
            (codepoint >= 0xd800 && codepoint <= 0xdfff)) return false;
    }
    return true;
}

bool valid_attributes(const std::vector<Attribute>& values) {
    for (const auto& attribute : values) {
        if (!valid_utf8(attribute.key) || !valid_utf8(attribute.value)) return false;
    }
    return true;
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
    return export_trace(resource, std::vector<Span>{span});
}

Status Telemetry::export_trace(const Resource& resource, const std::vector<Span>& spans) const {
    if (output_.empty()) return Status::Ok;
    if (spans.empty() || !valid_attributes(resource.attributes)) return Status::InvalidArgument;
    for (const auto& span : spans) {
        if (span.name.empty() || !valid_utf8(span.name) || !valid_attributes(span.attributes) ||
            !valid_id(span.ids.trace_id, 32) || !valid_id(span.ids.span_id, 16) ||
            span.end_unix_nano < span.start_unix_nano ||
            (!span.ids.parent_span_id.empty() && !valid_id(span.ids.parent_span_id, 16))) {
            return Status::InvalidArgument;
        }
    }
    std::ostringstream json;
    json << "{\"resourceSpans\":[{\"resource\":{\"attributes\":"
         << attributes(resource.attributes)
         << "},\"scopeSpans\":[{\"scope\":{\"name\":\"uor-cuda-inverted-file\"},\"spans\":[";
    for (std::size_t i = 0; i < spans.size(); ++i) {
        const auto& span = spans[i];
        if (i != 0) json << ',';
        json << "{\"traceId\":\"" << span.ids.trace_id
             << "\",\"spanId\":\"" << span.ids.span_id << "\",";
        if (!span.ids.parent_span_id.empty()) {
            json << "\"parentSpanId\":\"" << span.ids.parent_span_id << "\",";
        }
        json << "\"name\":\"" << escape(span.name)
             << "\",\"kind\":1,\"startTimeUnixNano\":\"" << span.start_unix_nano
             << "\",\"endTimeUnixNano\":\"" << span.end_unix_nano
             << "\",\"attributes\":" << attributes(span.attributes)
             << ",\"status\":{\"code\":0}}";
    }
    json << "]}]}]}";
    auto temporary = output_;
    temporary += ".tmp";
    {
        std::ofstream output(temporary, std::ios::trunc);
        if (!output) return Status::TelemetryFailure;
        output << json.str() << '\n';
        output.close();
        if (!output) {
            std::error_code ignored;
            std::filesystem::remove(temporary, ignored);
            return Status::TelemetryFailure;
        }
    }
    std::error_code error;
    std::filesystem::rename(temporary, output_, error);
    if (error) {
        std::error_code ignored;
        std::filesystem::remove(temporary, ignored);
        return Status::TelemetryFailure;
    }
    return Status::Ok;
}

} // namespace uor_inverted_file
