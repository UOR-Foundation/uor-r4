#include "../include/uor_inverted_file.hpp"

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <utility>
#include <vector>

namespace uor_inverted_file {
namespace {

std::uint32_t root_id(const Block& block) {
    for (std::uint32_t root = 0; root < codebook.size(); ++root) {
        if (block == codebook[root]) return root;
    }
    return ABSENT;
}

std::uint32_t token_cell(const Token& token) {
    const auto first = root_id(token[0]);
    const auto last = root_id(token[3]);
    if (first == ABSENT || last == ABSENT) return ABSENT;
    return first * 16 + last;
}

std::int64_t l1(const Query& query, const Token& token) {
    std::int64_t distance = 0;
    for (std::size_t block = 0; block < query.size(); ++block) {
        for (std::size_t d = 0; d < query[block].size(); ++d) {
            const auto delta = static_cast<std::int64_t>(query[block][d]) - token[block][d];
            distance += delta < 0 ? -delta : delta;
        }
    }
    return distance;
}

std::int64_t cell_distance(const Query& query, std::uint32_t cell) {
    std::int64_t distance = 0;
    const Block& first = codebook[cell / 16];
    const Block& last = codebook[cell % 16];
    for (std::size_t d = 0; d < first.size(); ++d) {
        const auto a = static_cast<std::int64_t>(query[0][d]) - first[d];
        const auto b = static_cast<std::int64_t>(query[3][d]) - last[d];
        distance += (a < 0 ? -a : a) + (b < 0 ? -b : b);
    }
    return distance;
}

void rank_tokens(const Table& table, const Query& query,
                 const std::array<std::uint32_t, 16>& admitted,
                 std::array<std::uint32_t, 8>& top8) {
    top8.fill(ABSENT);
    std::vector<std::pair<std::int64_t, std::uint32_t>> ranked;
    for (const auto cell : admitted) {
        if (cell == ABSENT) continue;
        for (const auto& token : table.tokens) {
            if (token_cell(token) != cell) continue;
            ranked.emplace_back(l1(query, token), &token - table.tokens.data());
        }
    }
    std::sort(ranked.begin(), ranked.end(), [](const auto& a, const auto& b) {
        return a.first != b.first ? a.first < b.first : a.second < b.second;
    });
    for (std::size_t i = 0; i < ranked.size() && i < top8.size(); ++i) {
        top8[i] = ranked[i].second;
    }
}

void fill_admitted(const Table&, const Query& query, const Index& index,
                   std::uint32_t k, std::array<std::uint32_t, 16>& admitted) {
    admitted.fill(ABSENT);
    std::vector<std::pair<std::int64_t, std::uint32_t>> cells;
    for (std::uint32_t cell = 0; cell < 256; ++cell) {
        if (index.offsets[cell] == index.offsets[cell + 1]) continue;
        cells.emplace_back(cell_distance(query, cell), cell);
    }
    std::sort(cells.begin(), cells.end(), [](const auto& a, const auto& b) {
        return a.first != b.first ? a.first < b.first : a.second < b.second;
    });
    for (std::uint32_t i = 0; i < k && i < cells.size(); ++i) admitted[i] = cells[i].second;
}

Status scan(const Table& table, const Query& query, std::uint32_t k,
            const std::array<std::uint32_t, 16>& admitted, ScanResult& result) {
    if (k == 0 || k > 16) return Status::InvalidArgument;
    rank_tokens(table, query, admitted, result.top8);
    result.prediction = result.top8[0];
    return Status::Ok;
}

} // namespace

Status validate(const Table& table, const Index& index) {
    if (index.cells.size() != table.tokens.size()) return Status::CsrInvariant;
    if (index.offsets[0] != 0 || index.offsets[256] != index.members.size()) {
        return Status::CsrInvariant;
    }
    std::uint32_t absent = 0;
    for (std::size_t token = 0; token < index.cells.size(); ++token) {
        const auto expected = token_cell(table.tokens[token]);
        if (index.cells[token] != expected) return Status::CsrInvariant;
        if (expected == ABSENT) ++absent;
    }
    if (absent != index.absent_count || absent + index.members.size() != table.tokens.size()) {
        return Status::CsrInvariant;
    }
    for (std::uint32_t cell = 0; cell < 256; ++cell) {
        if (index.offsets[cell] > index.offsets[cell + 1]) return Status::CsrInvariant;
        const auto begin = index.offsets[cell];
        const auto end = index.offsets[cell + 1];
        if (end > index.members.size()) return Status::CsrInvariant;
        std::uint32_t previous = 0;
        for (auto pos = begin; pos < end; ++pos) {
            const auto token = index.members[pos];
            if (token >= table.tokens.size() || token_cell(table.tokens[token]) != cell) {
                return Status::CsrInvariant;
            }
            if (pos > begin && previous >= token) return Status::CsrInvariant;
            previous = token;
        }
    }
    return Status::Ok;
}

Status build(const Table& table, Index& index) {
    if (table.tokens.empty()) return Status::InvalidArgument;
    index = Index{};
    index.cells.reserve(table.tokens.size());
    for (const auto& token : table.tokens) {
        const auto cell = token_cell(token);
        if (cell == ABSENT) return Status::NonRootBlock;
        index.cells.push_back(cell);
    }
    for (const auto cell : index.cells) ++index.offsets[cell + 1];
    for (std::uint32_t cell = 0; cell < 256; ++cell) {
        index.offsets[cell + 1] += index.offsets[cell];
    }
    std::array<std::uint32_t, 256> cursor{};
    for (std::uint32_t cell = 0; cell < 256; ++cell) cursor[cell] = index.offsets[cell];
    index.members.resize(index.cells.size());
    for (std::uint32_t token = 0; token < index.cells.size(); ++token) {
        index.members[cursor[index.cells[token]]++] = token;
    }
    return validate(table, index);
}

Status update(const Table& table, std::uint32_t moved_token, Index& index) {
    if (moved_token >= table.tokens.size() || index.cells.size() != table.tokens.size()) {
        return Status::InvalidArgument;
    }
    const auto old_cell = index.cells[moved_token];
    const auto new_cell = token_cell(table.tokens[moved_token]);
    if (old_cell == new_cell) return Status::Ok;
    auto& members = index.members;
    auto& offsets = index.offsets;
    std::uint32_t remove_pos = ABSENT;
    if (old_cell != ABSENT) {
        for (auto pos = offsets[old_cell]; pos < offsets[old_cell + 1]; ++pos) {
            if (members[pos] == moved_token) {
                remove_pos = pos;
                break;
            }
        }
        if (remove_pos == ABSENT) return Status::CsrInvariant;
    } else if (index.absent_count > 0) {
        --index.absent_count;
    }

    if (new_cell == ABSENT) {
        if (remove_pos == ABSENT) return Status::CsrInvariant;
        members.erase(members.begin() + static_cast<std::ptrdiff_t>(remove_pos));
        for (std::uint32_t cell = old_cell + 1; cell <= 256; ++cell) --offsets[cell];
        ++index.absent_count;
    } else if (old_cell == ABSENT) {
        auto pos = offsets[new_cell];
        while (pos < offsets[new_cell + 1] && members[pos] < moved_token) ++pos;
        members.insert(members.begin() + static_cast<std::ptrdiff_t>(pos), moved_token);
        for (std::uint32_t cell = new_cell + 1; cell <= 256; ++cell) ++offsets[cell];
    } else if (old_cell < new_cell) {
        auto insert_before = offsets[new_cell];
        while (insert_before < offsets[new_cell + 1] && members[insert_before] < moved_token) {
            ++insert_before;
        }
        const auto target = insert_before - 1;
        for (auto pos = remove_pos; pos < target; ++pos) members[pos] = members[pos + 1];
        members[target] = moved_token;
        for (std::uint32_t cell = old_cell + 1; cell <= new_cell; ++cell) --offsets[cell];
    } else {
        auto insert_pos = offsets[new_cell];
        while (insert_pos < offsets[new_cell + 1] && members[insert_pos] < moved_token) {
            ++insert_pos;
        }
        for (auto pos = remove_pos; pos > insert_pos; --pos) members[pos] = members[pos - 1];
        members[insert_pos] = moved_token;
        for (std::uint32_t cell = new_cell + 1; cell <= old_cell; ++cell) ++offsets[cell];
    }
    index.cells[moved_token] = new_cell;
    return validate(table, index);
}

Status probe(const Table& table, const Query& query, std::uint32_t k,
             const Index& index, ScanResult& result) {
    if (k == 0 || k > 16) return Status::InvalidArgument;
    if (validate(table, index) != Status::Ok) return Status::CsrInvariant;
    fill_admitted(table, query, index, k, result.admitted);
    return scan(table, query, k, result.admitted, result);
}

Status full_scan(const Table& table, const Query& query, std::uint32_t k,
                 ScanResult& result) {
    if (k == 0 || k > 16) return Status::InvalidArgument;
    Index index;
    const auto status = build(table, index);
    if (status != Status::Ok) return status;
    fill_admitted(table, query, index, k, result.admitted);
    return scan(table, query, k, result.admitted, result);
}

} // namespace uor_inverted_file
