#pragma once

#include "../include/uor_inverted_file.hpp"

#include <string_view>
#include <vector>

namespace uor_inverted_file {

[[nodiscard]] Status admit_ptx(
    std::string_view ptx,
    const std::vector<std::string_view>& entries);

} // namespace uor_inverted_file
