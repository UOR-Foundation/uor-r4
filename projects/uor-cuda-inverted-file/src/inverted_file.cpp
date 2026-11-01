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

void rank_bucket(const Table& table, const Query& query, const Index& index,
                 const std::array<std::uint32_t, 16>& admitted,
                 std::array<std::uint32_t, 8>& top8) {
    top8.fill(ABSENT);
    std::vector<std::pair<std::int64_t, std::uint32_t>> ranked;
    for (const auto cell : admitted) {
        if (cell == ABSENT) continue;
        for (auto pos = index.offsets[cell]; pos < index.offsets[cell + 1]; ++pos) {
            const auto token = index.members[pos];
            ranked.emplace_back(l1(query, table.tokens[token]), token);
        }
    }
    std::sort(ranked.begin(), ranked.end(), [](const auto& a, const auto& b) {
        return a.first != b.first ? a.first < b.first : a.second < b.second;
    });
    for (std::size_t i = 0; i < ranked.size() && i < top8.size(); ++i) top8[i] = ranked[i].second;
}

void rank_full_scan(const Table& table, const Query& query,
                    const std::array<std::uint32_t, 16>& admitted,
                    std::array<std::uint32_t, 8>& top8) {
    top8.fill(ABSENT);
    std::vector<std::pair<std::int64_t, std::uint32_t>> ranked;
    for (std::uint32_t token = 0; token < table.tokens.size(); ++token) {
        const auto cell = token_cell(table.tokens[token]);
        if (cell == ABSENT) continue;
        if (std::find(admitted.begin(), admitted.end(), cell) == admitted.end()) continue;
        ranked.emplace_back(l1(query, table.tokens[token]), token);
    }
    std::sort(ranked.begin(), ranked.end(), [](const auto& a, const auto& b) {
        return a.first != b.first ? a.first < b.first : a.second < b.second;
    });
    for (std::size_t i = 0; i < ranked.size() && i < top8.size(); ++i) top8[i] = ranked[i].second;
}

void fill_admitted(const Query& query, const Index& index, std::uint32_t k,
                   std::array<std::uint32_t, 16>& admitted) {
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

Status build_index(const Table& table, Index& index, bool allow_absent) {
    if (table.tokens.empty()) return Status::InvalidArgument;
    index = Index{};
    index.cells.reserve(table.tokens.size());
    for (const auto& token : table.tokens) {
        const auto cell = token_cell(token);
        if (cell == ABSENT && !allow_absent) return Status::NonRootBlock;
        index.cells.push_back(cell);
        if (cell == ABSENT) {
            ++index.absent_count;
        } else {
            ++index.offsets[cell + 1];
        }
    }
    for (std::uint32_t cell = 0; cell < 256; ++cell) {
        index.offsets[cell + 1] += index.offsets[cell];
    }
    std::array<std::uint32_t, 256> cursor{};
    for (std::uint32_t cell = 0; cell < 256; ++cell) cursor[cell] = index.offsets[cell];
    index.members.resize(table.tokens.size() - index.absent_count);
    for (std::uint32_t token = 0; token < index.cells.size(); ++token) {
        if (index.cells[token] != ABSENT) index.members[cursor[index.cells[token]]++] = token;
    }
    return validate(table, index);
}

Status remove_from_bucket(std::uint32_t old_cell, std::uint32_t moved_token, Index& index) {
    auto& members = index.members;
    auto& offsets = index.offsets;
    std::uint32_t remove_pos = ABSENT;
    for (auto pos = offsets[old_cell]; pos < offsets[old_cell + 1]; ++pos) {
        if (members[pos] == moved_token) {
            remove_pos = pos;
            break;
        }
    }
    if (remove_pos == ABSENT) return Status::CsrInvariant;
    members.erase(members.begin() + static_cast<std::ptrdiff_t>(remove_pos));
    for (std::uint32_t cell = old_cell + 1; cell <= 256; ++cell) --offsets[cell];
    ++index.absent_count;
    return Status::Ok;
}

} // namespace

Status validate_structure(const Index& index) {
    if (index.offsets[0] != 0 || index.offsets[256] != index.members.size()) {
        return Status::CsrInvariant;
    }
    if (index.absent_count + index.members.size() != index.cells.size()) {
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
            if (token >= index.cells.size()) return Status::CsrInvariant;
            if (pos > begin && previous >= token) return Status::CsrInvariant;
            previous = token;
        }
    }
    return Status::Ok;
}

Status validate(const Table& table, const Index& index) {
    if (index.cells.size() != table.tokens.size()) return Status::CsrInvariant;
    if (validate_structure(index) != Status::Ok) return Status::CsrInvariant;
    std::uint32_t absent = 0;
    for (std::size_t token = 0; token < index.cells.size(); ++token) {
        if (index.cells[token] == ABSENT) {
            ++absent;
            continue;
        }
        if (index.cells[token] != token_cell(table.tokens[token])) return Status::CsrInvariant;
    }
    return absent == index.absent_count ? Status::Ok : Status::CsrInvariant;
}

Status build(const Table& table, Index& index, bool allow_absent) {
    return build_index(table, index, allow_absent);
}

Status update(const Table& table, std::uint32_t moved_token, Index& index) {
    if (moved_token >= table.tokens.size() || index.cells.size() != table.tokens.size()) {
        return Status::InvalidArgument;
    }
    if (validate_structure(index) != Status::Ok) return Status::CsrInvariant;
    const auto old_cell = index.cells[moved_token];
    const auto new_cell = token_cell(table.tokens[moved_token]);
    if (new_cell == ABSENT) return Status::NonRootBlock;
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
    } else if (index.absent_count == 0) {
        return Status::CsrInvariant;
    }
    if (old_cell == ABSENT) {
        auto pos = offsets[new_cell];
        while (pos < offsets[new_cell + 1] && members[pos] < moved_token) ++pos;
        members.insert(members.begin() + static_cast<std::ptrdiff_t>(pos), moved_token);
        for (std::uint32_t cell = new_cell + 1; cell <= 256; ++cell) ++offsets[cell];
        --index.absent_count;
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

Status remove_token(const Table& table, std::uint32_t moved_token, Index& index) {
    if (moved_token >= table.tokens.size() || index.cells.size() != table.tokens.size()) {
        return Status::InvalidArgument;
    }
    if (validate_structure(index) != Status::Ok) return Status::CsrInvariant;
    const auto old_cell = index.cells[moved_token];
    if (old_cell == ABSENT) return Status::Ok;
    const auto status = remove_from_bucket(old_cell, moved_token, index);
    if (status != Status::Ok) return status;
    index.cells[moved_token] = ABSENT;
    return validate_structure(index);
}

Status probe(const Table& table, const Query& query, std::uint32_t k,
             const Index& index, ScanResult& result) {
    if (k == 0 || k > 16) return Status::InvalidArgument;
    if (validate(table, index) != Status::Ok) return Status::CsrInvariant;
    fill_admitted(query, index, k, result.admitted);
    rank_bucket(table, query, index, result.admitted, result.top8);
    result.prediction = result.top8[0];
    return result.prediction == ABSENT ? Status::InvalidArgument : Status::Ok;
}

Status full_scan(const Table& table, const Query& query, std::uint32_t k,
                 ScanResult& result) {
    if (k == 0 || k > 16) return Status::InvalidArgument;
    Index index;
    const auto status = build_index(table, index, true);
    if (status != Status::Ok) return status;
    fill_admitted(query, index, k, result.admitted);
    rank_full_scan(table, query, result.admitted, result.top8);
    result.prediction = result.top8[0];
    return result.prediction == ABSENT ? Status::InvalidArgument : Status::Ok;
}

} // namespace uor_inverted_file
