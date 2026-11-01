#pragma once

#include <string>

namespace uor_inverted_file {

struct SpanIds {
    std::string trace_id;
    std::string span_id;
    std::string parent_span_id;
};

[[nodiscard]] std::string new_trace_id();
[[nodiscard]] std::string new_span_id();

} // namespace uor_inverted_file
