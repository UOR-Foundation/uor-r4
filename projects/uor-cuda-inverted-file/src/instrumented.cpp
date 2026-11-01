#include "instrumented.hpp"

#include "sha256.hpp"

#include <chrono>
#include <string>

namespace uor_inverted_file {
namespace {

std::uint64_t now_ns() {
    const auto elapsed = std::chrono::system_clock::now().time_since_epoch();
    return static_cast<std::uint64_t>(std::chrono::duration_cast<std::chrono::nanoseconds>(elapsed).count());
}

std::string status_name(Status status) {
    switch (status) {
        case Status::Ok: return "ok";
        case Status::InvalidArgument: return "invalid-argument";
        case Status::NonRootBlock: return "non-root-block";
        case Status::CsrInvariant: return "csr-invariant";
        case Status::CudaUnavailable: return "cuda-unavailable";
        case Status::CudaFailure: return "cuda-failure";
        case Status::TelemetryFailure: return "telemetry-failure";
    }
    return "unknown";
}

Resource resource_for_codebook() {
    const auto byte_size = codebook.size() * sizeof(Block);
    const auto digest = sha256_hex(reinterpret_cast<const std::uint8_t*>(codebook.data()), byte_size);
    return Resource{{
        {"uor.schema.version", "uor.telemetry/1"},
        {"uor.artifact.payload_sha256", digest},
        {"uor.artifact.digest.algorithm", "sha256"},
        {"uor.artifact.identity_kind", "fixed-codebook"},
        {"uor.artifact.byte_size", std::to_string(byte_size)},
        {"uor.serving.kernel", "cpu-oracle"},
    }};
}

} // namespace

Instrumented::Instrumented(std::filesystem::path output) : telemetry_(std::move(output)) {}

bool Instrumented::enabled() const { return telemetry_.enabled(); }

Status Instrumented::emit(Span span) {
    if (root_trace_id_.empty()) root_trace_id_ = new_trace_id();
    span.ids.trace_id = root_trace_id_;
    span.ids.span_id = new_span_id();
    if (root_span_id_.empty()) {
        root_span_id_ = span.ids.span_id;
    } else {
        span.ids.parent_span_id = root_span_id_;
    }
    spans_.push_back(std::move(span));
    return telemetry_.export_trace(resource_for_codebook(), spans_);
}

Status Instrumented::build(const Table& table, Index& index) {
    const auto start = now_ns();
    const auto status = uor_inverted_file::build(table, index);
    Span span{"uor.cuda.index.build", SpanIds{}, start, now_ns(),
              {{"uor.operation.status", status_name(status)}}};
    const auto telemetry_status = emit(std::move(span));
    return status == Status::Ok ? telemetry_status : status;
}

Status Instrumented::update(const Table& table, std::uint32_t moved_token, Index& index) {
    const auto start = now_ns();
    const auto status = uor_inverted_file::update(table, moved_token, index);
    Span span{"uor.cuda.index.update", SpanIds{}, start, now_ns(),
              {{"uor.operation.status", status_name(status)},
               {"uor.operation.moved_token", std::to_string(moved_token)}}};
    const auto telemetry_status = emit(std::move(span));
    return status == Status::Ok ? telemetry_status : status;
}

Status Instrumented::probe(const Table& table, const Query& query, std::uint32_t k,
                           const Index& index, ScanResult& result) {
    const auto start = now_ns();
    const auto status = uor_inverted_file::probe(table, query, k, index, result);
    Span span{"uor.cuda.index.probe", SpanIds{}, start, now_ns(),
              {{"uor.operation.status", status_name(status)},
               {"uor.operation.k", std::to_string(k)}}};
    const auto telemetry_status = emit(std::move(span));
    return status == Status::Ok ? telemetry_status : status;
}

Status Instrumented::full_scan(const Table& table, const Query& query, std::uint32_t k,
                               ScanResult& result) {
    const auto start = now_ns();
    const auto status = uor_inverted_file::full_scan(table, query, k, result);
    Span span{"uor.cuda.index.full_scan", SpanIds{}, start, now_ns(),
              {{"uor.operation.status", status_name(status)},
               {"uor.operation.k", std::to_string(k)}}};
    const auto telemetry_status = emit(std::move(span));
    return status == Status::Ok ? telemetry_status : status;
}

} // namespace uor_inverted_file
