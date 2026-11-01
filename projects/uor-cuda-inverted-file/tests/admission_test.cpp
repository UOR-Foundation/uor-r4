#include "../src/admission.hpp"

#include <iostream>
#include <string>
#include <string_view>
#include <vector>

namespace {

int failures = 0;

void expect(bool condition, const char* name) {
    if (!condition) {
        ++failures;
        std::cerr << "FAIL: " << name << '\n';
    }
}

std::string entry(std::string_view name) {
    return ".visible .entry " + std::string(name) + "() { ret; }\n";
}

const std::vector<std::string_view> entries{
    "uor_if_build", "uor_if_update", "uor_if_probe", "uor_if_full_scan"};

std::string clean_module() {
    return entry("uor_if_build") + entry("uor_if_update") +
           entry("uor_if_probe") + entry("uor_if_full_scan");
}

} // namespace

int main() {
    using uor_inverted_file::admit_ptx;
    using uor_inverted_file::Status;

    expect(admit_ptx(clean_module(), entries) == Status::Ok, "clean four-entry module");
    expect(admit_ptx("// add.f32 %r1,%r2,%r3;\n" + clean_module(), entries) == Status::Ok,
           "float instruction in a comment is ignored");
    expect(admit_ptx("// .reg .bf16 %r1;\n" + clean_module(), entries) == Status::Ok,
           "float register in a comment is ignored");
    expect(admit_ptx("add.f32 %r1,%r2,%r3;\n" + clean_module(), entries) == Status::CudaFailure,
           "f32 instruction rejected");
    expect(admit_ptx(".reg .bf16 %r1;\n" + clean_module(), entries) == Status::CudaFailure,
           "bf16 register rejected");
    expect(admit_ptx("mma.sync.aligned.m16n8k16.row.col.f32.f16.f16.f32\n" + clean_module(), entries) == Status::CudaFailure,
           "tensor instruction rejected");
    expect(admit_ptx("ex2.approx.f32 %r1,%r2;\n" + clean_module(), entries) == Status::CudaFailure,
           "approximate exponential rejected");
    expect(admit_ptx(entry("uor_if_build") + entry("uor_if_update") + entry("uor_if_probe"), entries) == Status::CudaFailure,
           "missing entry rejected");
    expect(admit_ptx(clean_module() + entry("uor_if_extra"), entries) == Status::CudaFailure,
           "extra entry rejected");
    expect(admit_ptx(clean_module() + entry("uor_if_build"), entries) == Status::CudaFailure,
           "duplicate entry rejected");
    expect(admit_ptx("/* add.f32 %r1,%r2,%r3; */\n" + clean_module(), entries) == Status::Ok,
           "block comment is ignored");
    expect(admit_ptx(".file \"a//b\";\nadd.f32 %r1,%r2,%r3;\n" + clean_module(), entries) == Status::CudaFailure,
           "line-comment markers inside strings do not hide instructions");
    expect(admit_ptx(".file \"a/*b*/\";\nadd.f32 %r1,%r2,%r3;\n" + clean_module(), entries) == Status::CudaFailure,
           "block-comment markers inside strings do not hide instructions");

    if (failures != 0) {
        std::cerr << failures << " admission assertions failed\n";
        return 1;
    }
    std::cout << "admission tests: 13/13 passed\n";
    return 0;
}
