#pragma once

#include "telemetry.hpp"

#include <chrono>

namespace uor_inverted_file {

[[nodiscard]] std::uint64_t telemetry_timestamp_after(std::uint64_t start,
                                                      std::chrono::nanoseconds elapsed);

class Instrumented {
public:
    explicit Instrumented(std::filesystem::path output = {});

    [[nodiscard]] bool enabled() const;
    [[nodiscard]] Status build(const Table& table, Index& index);
    [[nodiscard]] Status update(const Table& table, std::uint32_t moved_token, Index& index);
    [[nodiscard]] Status probe(const Table& table, const Query& query, std::uint32_t k,
                               const Index& index, ScanResult& result);
    [[nodiscard]] Status full_scan(const Table& table, const Query& query, std::uint32_t k,
                                   ScanResult& result);

private:
    [[nodiscard]] Status emit(Span span);

    Telemetry telemetry_;
    std::string root_trace_id_;
    std::string root_span_id_;
    std::vector<Span> spans_;
};

} // namespace uor_inverted_file
