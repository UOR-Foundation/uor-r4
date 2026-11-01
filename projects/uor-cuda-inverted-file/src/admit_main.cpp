#include "admission.hpp"

#include <fstream>
#include <iostream>
#include <iterator>
#include <string>
#include <vector>

int main(int argc, char** argv) {
    if (argc != 2) {
        std::cerr << "usage: admit <module.ptx>\n";
        return 2;
    }
    std::ifstream input(argv[1]);
    if (!input) {
        std::cerr << "cannot read " << argv[1] << '\n';
        return 2;
    }
    const std::string ptx((std::istreambuf_iterator<char>(input)), std::istreambuf_iterator<char>());
    const std::vector<std::string_view> entries{
        "uor_if_build", "uor_if_update", "uor_if_probe", "uor_if_full_scan"};
    const auto status = uor_inverted_file::admit_ptx(ptx, entries);
    if (status != uor_inverted_file::Status::Ok) {
        std::cerr << "PTX admission failed\n";
        return 1;
    }
    std::cout << "PTX admission passed: 4/4 entries\n";
    return 0;
}
