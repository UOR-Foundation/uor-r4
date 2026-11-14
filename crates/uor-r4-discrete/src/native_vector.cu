// Integer-only Q31 Hamilton transport for the native vector trainer.
// No float, matrix, transcendental or runtime-compilation path exists here.
typedef long long s64;
typedef unsigned long long u64;

__device__ __forceinline__ u64 product(s64 a, s64 b) {
    return (u64)(a * b);
}

__device__ __forceinline__ s64 q31_word(u64 word) {
    return (s64)(word + (1ull << 30)) >> 31;
}

extern "C" __global__ void native_q31_hamilton_rotate(
    const s64* gates,
    const s64* values,
    s64* output,
    unsigned count
) {
    unsigned item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= count) return;
    const s64* a = gates + item * 4;
    const s64* b = values + item * 4;
    s64 p00 = a[0], p01 = a[0], p02 = a[0], p03 = a[0];
    s64 p10 = a[1], p11 = a[1], p12 = a[1], p13 = a[1];
    s64 p20 = a[2], p21 = a[2], p22 = a[2], p23 = a[2];
    s64 p30 = a[3], p31 = a[3], p32 = a[3], p33 = a[3];
    u64 w =
        product(p00, b[0]) -
        product(p11, b[1]) -
        product(p22, b[2]) -
        product(p33, b[3]);
    u64 x =
        product(p01, b[1]) +
        product(p10, b[0]) +
        product(p23, b[3]) -
        product(p32, b[2]);
    u64 y =
        product(p02, b[2]) -
        product(p13, b[3]) +
        product(p20, b[0]) +
        product(p31, b[1]);
    u64 z =
        product(p03, b[3]) +
        product(p12, b[2]) -
        product(p21, b[1]) +
        product(p30, b[0]);
    output[item * 4] = q31_word(w);
    output[item * 4 + 1] = q31_word(x);
    output[item * 4 + 2] = q31_word(y);
    output[item * 4 + 3] = q31_word(z);
}
