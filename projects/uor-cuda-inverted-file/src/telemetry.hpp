#pragma once

#include "../include/uor_inverted_file.hpp"
#include "telemetry_ids.hpp"

#include <filesystem>
#include <string>
#include <vector>

namespace uor_inverted_file {

struct Attribute {
    std::string key;
    std::string value;
};

struct Resource {
    std::vector<Attribute> attributes;
};

struct Span {
    std::string name;
    SpanIds ids;
    std::uint64_t start_unix_nano = 0;
    std::uint64_t end_unix_nano = 0;
    std::vector<Attribute> attributes;
};

class Telemetry {
public:
    explicit Telemetry(std::filesystem::path output = {});

    [[nodiscard]] bool enabled() const;
    [[nodiscard]] Status export_trace(const Resource& resource, const Span& span) const;
    [[nodiscard]] Status export_trace(const Resource& resource, const std::vector<Span>& spans) const;

private:
    std::filesystem::path output_;
};

} // namespace uor_inverted_file
