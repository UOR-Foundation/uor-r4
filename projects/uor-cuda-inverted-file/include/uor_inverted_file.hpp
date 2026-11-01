#pragma once

#include <array>
#include <cstdint>
#include <limits>
#include <vector>

namespace uor_inverted_file {

using Block = std::array<std::int32_t, 8>;
using Token = std::array<Block, 4>;
using Query = std::array<Block, 4>;
constexpr std::uint32_t ABSENT = std::numeric_limits<std::uint32_t>::max();

struct Table {
    std::vector<Token> tokens;
};

struct Index {
    std::array<std::uint32_t, 257> offsets{};
    std::vector<std::uint32_t> members;
    std::vector<std::uint32_t> cells;
    std::uint32_t absent_count = 0;
};

struct ScanResult {
    std::uint32_t prediction = ABSENT;
    std::array<std::uint32_t, 16> admitted{};
    std::array<std::uint32_t, 8> top8{};
};

enum class Status {
    Ok,
    InvalidArgument,
    NonRootBlock,
    CsrInvariant,
    CudaUnavailable,
    CudaFailure,
};

extern const std::array<Block, 16> codebook;

[[nodiscard]] Status build(const Table& table, Index& index, bool allow_absent = false);
[[nodiscard]] Status update(const Table& table, std::uint32_t moved_token, Index& index);
[[nodiscard]] Status remove_token(const Table& table, std::uint32_t moved_token, Index& index);
[[nodiscard]] Status probe(const Table& table, const Query& query, std::uint32_t k,
                           const Index& index, ScanResult& result);
[[nodiscard]] Status full_scan(const Table& table, const Query& query, std::uint32_t k,
                               ScanResult& result);
[[nodiscard]] Status validate(const Table& table, const Index& index);
[[nodiscard]] Status validate_structure(const Index& index);

} // namespace uor_inverted_file
