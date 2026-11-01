#pragma once

#include "../include/uor_inverted_file.hpp"

namespace uor_inverted_file::cuda {

[[nodiscard]] Status build(const Table& table, Index& index);
[[nodiscard]] Status update(const Table& table, std::uint32_t moved_token, Index& index);
[[nodiscard]] Status probe(const Table& table, const Query& query, std::uint32_t k,
                           const Index& index, ScanResult& result);
[[nodiscard]] Status full_scan(const Table& table, const Query& query, std::uint32_t k,
                               ScanResult& result);

} // namespace uor_inverted_file::cuda
