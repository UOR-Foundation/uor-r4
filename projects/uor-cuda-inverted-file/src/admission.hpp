#pragma once

#include <string_view>
#include <vector>

namespace uor_inverted_file {

enum class Status {
    Ok,
    InvalidArgument,
    NonRootBlock,
    CsrInvariant,
    CudaUnavailable,
    CudaFailure,
};

[[nodiscard]] Status admit_ptx(
    std::string_view ptx,
    const std::vector<std::string_view>& entries);

} // namespace uor_inverted_file
