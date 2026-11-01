#include "admission.hpp"

#include <algorithm>
#include <array>
#include <sstream>
#include <string>
#include <vector>

namespace uor_inverted_file {
namespace {

std::string without_comments(std::string_view input) {
    std::string output;
    output.reserve(input.size());
    enum class Comment { None, Line, Block } state = Comment::None;
    for (std::size_t i = 0; i < input.size(); ++i) {
        const char c = input[i];
        const char next = i + 1 < input.size() ? input[i + 1] : '\0';
        if (state == Comment::None && c == '/' && next == '/') {
            state = Comment::Line;
            ++i;
            continue;
        }
        if (state == Comment::None && c == '/' && next == '*') {
            state = Comment::Block;
            ++i;
            continue;
        }
        if (state == Comment::Line && c == '\n') {
            state = Comment::None;
            output.push_back(c);
            continue;
        }
        if (state == Comment::Block && c == '*' && next == '/') {
            state = Comment::None;
            ++i;
            continue;
        }
        if (state == Comment::None) output.push_back(c);
    }
    return output;
}

bool forbidden_token(std::string_view token) {
    static constexpr std::array<std::string_view, 17> forbidden{
        "f16", "f16x2", "f32", "f64", "bf16", "bf16x2", "tf32", "e4m3", "e5m2",
        "mma", "wmma", "wgmma", "tcgen05", "tensor", "tensormap", "ex2", "approx"};
    if (token.compare(0, 10, "ex2.approx") == 0) return true;
    std::size_t begin = 0;
    while (begin < token.size()) {
        const std::size_t end = token.find_first_of(".;,()[]{}", begin);
        const auto part = token.substr(begin, end == std::string_view::npos ? end : end - begin);
        if (std::find(forbidden.begin(), forbidden.end(), part) != forbidden.end()) return true;
        if (end == std::string_view::npos) break;
        begin = end + 1;
    }
    return false;
}

bool forbidden_instruction(std::string_view line) {
    std::istringstream stream{std::string(line)};
    std::string token;
    while (stream >> token) {
        if (forbidden_token(token)) return true;
    }
    return false;
}

} // namespace

Status admit_ptx(std::string_view ptx, const std::vector<std::string_view>& entries) {
    if (entries.empty()) return Status::InvalidArgument;
    const std::string clean = without_comments(ptx);
    std::vector<std::string> declared;
    std::istringstream lines(clean);
    std::string line;
    while (std::getline(lines, line)) {
        if (forbidden_instruction(line)) return Status::CudaFailure;
        const auto first = line.find_first_not_of(" \t");
        if (first == std::string::npos) continue;
        const std::string_view trimmed(line.data() + first, line.size() - first);
        std::size_t prefix_length = 0;
        if (trimmed.compare(0, 16, ".visible .entry ") == 0) {
            prefix_length = 16;
        } else if (trimmed.compare(0, 7, ".entry ") == 0) {
            prefix_length = 7;
        }
        if (prefix_length == 0) continue;
        const auto rest = trimmed.substr(prefix_length);
        const auto end = rest.find_first_of(" (");
        declared.emplace_back(rest.substr(0, end));
    }
    if (declared.size() != entries.size()) return Status::CudaFailure;
    std::vector<std::string> actual = declared;
    std::vector<std::string_view> expected(entries.begin(), entries.end());
    std::sort(actual.begin(), actual.end());
    std::sort(expected.begin(), expected.end());
    if (std::adjacent_find(actual.begin(), actual.end()) != actual.end()) {
        return Status::CudaFailure;
    }
    if (!std::equal(actual.begin(), actual.end(), expected.begin(), expected.end())) {
        return Status::CudaFailure;
    }
    return Status::Ok;
}

} // namespace uor_inverted_file
