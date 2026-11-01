#include "../include/uor_inverted_file.hpp"

#include <algorithm>
#include <array>
#include <cstdint>
#include <iostream>
#include <limits>
#include <vector>

namespace {

using uor_inverted_file::Block;
using uor_inverted_file::Index;
using uor_inverted_file::Query;
using uor_inverted_file::ScanResult;
using uor_inverted_file::Status;
using uor_inverted_file::Table;
using uor_inverted_file::Token;
constexpr uint32_t absent = std::numeric_limits<uint32_t>::max();
int failures = 0;

void expect(bool condition, const char* name) {
    if (!condition) {
        ++failures;
        std::cerr << "FAIL: " << name << '\n';
    }
}

const std::array<Block, 16> roots{{
    {2, 2, 0, 0, 0, 0, 0, 0}, {-2, -2, 0, 0, 0, 0, 0, 0},
    {-2, 2, 0, 0, 0, 0, 0, 0}, {0, 0, -2, 0, -2, 0, 0, 0},
    {0, 0, -2, 0, 2, 0, 0, 0}, {0, 0, 0, -2, 0, 0, 0, -2},
    {0, 0, 0, -2, 0, 0, 0, 2}, {0, 0, 0, 0, 0, -2, -2, 0},
    {0, 0, 0, 0, 0, -2, 2, 0}, {0, 0, 0, 0, 0, 2, -2, 0},
    {0, 0, 0, 0, 0, 2, 2, 0}, {0, 0, 0, 2, 0, 0, 0, -2},
    {0, 0, 0, 2, 0, 0, 0, 2}, {0, 0, 2, 0, -2, 0, 0, 0},
    {0, 0, 2, 0, 2, 0, 0, 0}, {2, -2, 0, 0, 0, 0, 0, 0},
}};

Token token_at(uint32_t i) {
    Token token{roots[(i * 7 + 1) % 16], Block{}, Block{}, roots[(i * 11 + 3) % 16]};
    for (uint32_t d = 0; d < 8; ++d) {
        token[1][d] = static_cast<int32_t>((i * 29 + d * 7) % 37) - 18;
        token[2][d] = static_cast<int32_t>((i * 13 + d * 5) % 41) - 20;
    }
    return token;
}

Table table_of(uint32_t count) {
    Table table;
    for (uint32_t i = 0; i < count; ++i) table.tokens.push_back(token_at(i));
    return table;
}

Table zero_table_of(uint32_t count) {
    Table table;
    for (uint32_t i = 0; i < count; ++i) {
        table.tokens.push_back(Token{roots[(i * 7 + 1) % 16], Block{}, Block{}, roots[(i * 11 + 3) % 16]});
    }
    return table;
}

Query query_at(uint32_t seed) {
    Query query{roots[seed % 16], Block{}, Block{}, roots[(seed * 5 + 1) % 16]};
    for (uint32_t d = 0; d < 8; ++d) {
        query[1][d] = static_cast<int32_t>((seed * 11 + d * 3) % 23) - 11;
        query[2][d] = static_cast<int32_t>((seed * 17 + d * 2) % 19) - 9;
    }
    return query;
}

Query zero_query_at(uint32_t seed) {
    return Query{roots[seed % 16], Block{}, Block{}, roots[(seed * 5 + 1) % 16]};
}

uint32_t root_id(const Block& block) {
    for (uint32_t root = 0; root < roots.size(); ++root) if (block == roots[root]) return root;
    return absent;
}

uint32_t cell_of(const Token& token) {
    const auto first = root_id(token[0]);
    const auto last = root_id(token[3]);
    return first == absent || last == absent ? absent : first * 16 + last;
}

int64_t distance(const Query& query, const Token& token) {
    int64_t distance = 0;
    for (size_t block = 0; block < query.size(); ++block) {
        for (size_t d = 0; d < query[block].size(); ++d) {
            const auto delta = static_cast<int64_t>(query[block][d]) - token[block][d];
            distance += delta < 0 ? -delta : delta;
        }
    }
    return distance;
}

void expect_valid_result(const Table& table, const Query& query,
                         const ScanResult& result, const char* name) {
    expect(result.prediction < table.tokens.size(), name);
    std::array<bool, 256> seen{};
    std::array<bool, 256> occupied{};
    for (const auto& token : table.tokens) {
        const auto cell = cell_of(token);
        if (cell != absent) occupied[cell] = true;
    }
    for (const auto cell : result.admitted) {
        if (cell == absent) continue;
        expect(cell < 256 && occupied[cell], name);
        expect(!seen[cell], name);
        seen[cell] = true;
    }
    int64_t previous_distance = -1;
    uint32_t previous_token = 0;
    for (const auto token : result.top8) {
        if (token == absent) continue;
        expect(token < table.tokens.size(), name);
        const auto current = distance(query, table.tokens[token]);
        expect(current >= previous_distance, name);
        if (current == previous_distance) expect(token > previous_token, name);
        previous_distance = current;
        previous_token = token;
    }
}

void expect_same(const ScanResult& a, const ScanResult& b, const char* name) {
    expect(a.prediction == b.prediction, name);
    expect(a.admitted == b.admitted, name);
    expect(a.top8 == b.top8, name);
}

void parity(const Table& table, uint32_t k, const char* name) {
    Index index;
    expect(uor_inverted_file::build(table, index) == Status::Ok, name);
    expect(uor_inverted_file::validate(table, index) == Status::Ok, name);
    ScanResult bucket, full;
    expect(uor_inverted_file::probe(table, query_at(7), k, index, bucket) == Status::Ok, name);
    expect(uor_inverted_file::full_scan(table, query_at(7), k, full) == Status::Ok, name);
    expect_same(bucket, full, name);
    expect_valid_result(table, query_at(7), bucket, name);
}

} // namespace

int main() {
    for (const uint32_t v : {1u, 64u, 256u, 4096u}) {
        for (const uint32_t k : {1u, 8u, 16u}) parity(table_of(v), k, "seeded table parity");
    }

    Table single;
    for (uint32_t i = 0; i < 256; ++i) single.tokens.push_back(Token{roots[0], Block{}, Block{}, roots[0]});
    parity(single, 8, "single-cell collapse");

    Table few;
    for (uint32_t i = 0; i < 3; ++i) few.tokens.push_back(Token{roots[i], Block{}, Block{}, roots[i]});
    parity(few, 16, "fewer than K occupied cells");

    Index index;
    expect(uor_inverted_file::build(few, index) == Status::Ok, "few-cell build");
    ScanResult result;
    expect(uor_inverted_file::probe(few, query_at(0), 0, index, result) == Status::InvalidArgument,
           "zero K refused");
    expect(uor_inverted_file::probe(few, query_at(0), 17, index, result) == Status::InvalidArgument,
           "K above 16 refused");

    Table invalid = few;
    invalid.tokens[0][0] = Block{};
    Index refused;
    expect(uor_inverted_file::build(invalid, refused) == Status::NonRootBlock,
           "non-root endpoint refused");

    Table moving = table_of(256);
    Index updated;
    expect(uor_inverted_file::build(moving, updated) == Status::Ok, "initial update-index build");
    const auto original = updated;
    expect(uor_inverted_file::update(moving, 0, updated) == Status::Ok, "no-op update");
    expect(updated.offsets == original.offsets && updated.members == original.members &&
               updated.cells == original.cells && updated.absent_count == original.absent_count,
           "no-op preserves index");
    for (uint32_t step = 0; step < 64; ++step) {
        const uint32_t moved = (step * 17) % 256;
        moving.tokens[moved][0] = roots[(step + 3) % 16];
        moving.tokens[moved][3] = roots[(step * 5 + 1) % 16];
        expect(uor_inverted_file::update(moving, moved, updated) == Status::Ok, "64 incremental updates");
        expect(uor_inverted_file::validate(moving, updated) == Status::Ok, "update CSR invariant");
        Index fresh;
        expect(uor_inverted_file::build(moving, fresh) == Status::Ok, "fresh build after update");
        expect(updated.offsets == fresh.offsets, "update offsets equal fresh build");
        expect(updated.members == fresh.members, "update members equal fresh build");
        expect(updated.cells == fresh.cells, "update cells equal fresh build");
        expect(updated.absent_count == fresh.absent_count, "update absent count equal fresh build");
        ScanResult a, b;
        expect(uor_inverted_file::probe(moving, query_at(step), 8, updated, a) == Status::Ok, "updated probe");
        expect(uor_inverted_file::full_scan(moving, query_at(step), 8, b) == Status::Ok, "updated full scan");
        expect_same(a, b, "updated parity");
    }

    Table removal = few;
    Index removal_index;
    expect(uor_inverted_file::build(removal, removal_index) == Status::Ok, "removal initial build");
    removal.tokens[0][0] = Block{};
    expect(uor_inverted_file::update(removal, 0, removal_index) == Status::NonRootBlock,
           "non-root update refused without mutation");
    expect(uor_inverted_file::remove_token(removal, 0, removal_index) == Status::Ok, "explicit removal");
    expect(removal_index.absent_count == 1, "ABSENT count after removal");
    expect(uor_inverted_file::validate(removal, removal_index) == Status::Ok, "removal CSR invariant");
    removal.tokens[0][0] = roots[0];
    expect(uor_inverted_file::update(removal, 0, removal_index) == Status::Ok, "ABSENT to cell update");
    expect(removal_index.absent_count == 0, "ABSENT count after reinsertion");
    expect(uor_inverted_file::validate(removal, removal_index) == Status::Ok, "reinsertion CSR invariant");

    Table drained = few;
    Index drained_index;
    expect(uor_inverted_file::build(drained, drained_index) == Status::Ok, "all-absent initial build");
    for (auto& token : drained.tokens) token[0] = Block{};
    for (uint32_t token = 0; token < drained.tokens.size(); ++token) {
        expect(uor_inverted_file::remove_token(drained, token, drained_index) == Status::Ok,
               "explicitly drain every token");
    }
    ScanResult drained_result;
    expect(uor_inverted_file::probe(drained, query_at(0), 1, drained_index, drained_result) == Status::InvalidArgument,
           "all-absent probe refused");
    expect(uor_inverted_file::full_scan(drained, query_at(0), 1, drained_result) == Status::InvalidArgument,
           "all-absent full scan refused");

    Table ties;
    ties.tokens.push_back(Token{roots[0], Block{}, Block{}, roots[0]});
    ties.tokens.push_back(Token{roots[0], Block{}, Block{}, roots[0]});
    parity(ties, 1, "token-id tie break");
    Index tie_index;
    expect(uor_inverted_file::build(ties, tie_index) == Status::Ok, "tie build");
    ScanResult tie_result;
    expect(uor_inverted_file::probe(ties, query_at(0), 1, tie_index, tie_result) == Status::Ok, "tie probe");
    expect(tie_result.prediction == 0, "lowest token id wins tie");

    Index stale = tie_index;
    stale.cells[0] = 7;
    expect(uor_inverted_file::validate(ties, stale) == Status::CsrInvariant, "stale bucket detected");

    Index corrupt = tie_index;
    corrupt.offsets[1] = 99;
    expect(uor_inverted_file::validate(ties, corrupt) == Status::CsrInvariant, "corrupted CSR detected");
    expect(uor_inverted_file::probe(ties, query_at(0), 1, corrupt, tie_result) == Status::CsrInvariant,
           "corrupted probe refused");
    expect(uor_inverted_file::update(ties, 0, corrupt) == Status::CsrInvariant,
           "corrupted update refused");
    Index out_of_range = tie_index;
    out_of_range.cells[0] = 999;
    expect(uor_inverted_file::update(ties, 0, out_of_range) == Status::CsrInvariant,
           "out-of-range old cell refused");

    Table over = few;
    Index over_index;
    expect(uor_inverted_file::build(over, over_index) == Status::Ok, "over-prune fixture build");
    ScanResult bucket, full;
    expect(uor_inverted_file::probe(over, query_at(0), 1, over_index, bucket) == Status::Ok, "over-prune probe");
    expect(uor_inverted_file::full_scan(over, query_at(0), 1, full) == Status::Ok, "over-prune full scan");
    bucket.admitted[0] = absent;
    expect(bucket.prediction != full.prediction || bucket.admitted != full.admitted || bucket.top8 != full.top8,
           "over-pruned result detected");
    auto reversed = bucket;
    std::reverse(reversed.top8.begin(), reversed.top8.end());
    expect(reversed.top8 != full.top8, "reversed tie order detected");

    uint32_t top8_changes = 0;
    uint32_t prediction_changes = 0;
    for (const uint32_t v : {64u, 256u}) {
        const Table varied_table = table_of(v);
        const Table zero_table = zero_table_of(v);
        for (const uint32_t k : {1u, 8u, 16u}) {
            for (uint32_t seed = 0; seed < 4; ++seed) {
                ScanResult varied, zero;
                expect(uor_inverted_file::full_scan(varied_table, query_at(seed), k, varied) == Status::Ok,
                       "varied-middle sensitivity probe");
                expect(uor_inverted_file::full_scan(zero_table, zero_query_at(seed), k, zero) == Status::Ok,
                       "zero-middle sensitivity control");
                if (varied.top8 != zero.top8) ++top8_changes;
                if (varied.prediction != zero.prediction) ++prediction_changes;
            }
        }
    }
    expect(top8_changes >= 3, "middle-block content changes at least three top-8 rows");
    expect(prediction_changes >= 1, "middle-block content changes at least one prediction");

    Table extreme = table_of(2);
    for (auto& token : extreme.tokens) {
        for (size_t block = 1; block < 3; ++block) token[block].fill(std::numeric_limits<int32_t>::max());
    }
    auto extreme_query = query_at(0);
    for (size_t block = 1; block < 3; ++block) extreme_query[block].fill(std::numeric_limits<int32_t>::min());
    parity(extreme, 1, "64-bit extreme L1 distance parity");

    if (failures != 0) {
        std::cerr << failures << " inverted-file assertions failed\n";
        return 1;
    }
    std::cout << "inverted-file tests: all parity, update, refusal, and fault checks passed\n";
    return 0;
}
