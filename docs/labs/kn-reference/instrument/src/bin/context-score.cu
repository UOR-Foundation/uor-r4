// AOT CUDA backend for fitting and replaying the native suffix count mixture.
#define main order_one_main
#include "count-score.cu"
#undef main
#include <array>

constexpr unsigned DEPTH = 4, COMPONENTS = DEPTH + 1, FLOOR = 8;
constexpr size_t HEADER = (V + COMPONENTS) * 4, ROW_BYTES = 12;
constexpr unsigned NO_ROW = UINT32_MAX;

struct Index {
    const U64 *keys;
    const U32 *starts;
    const ID *followers;
    const U32 *table;
    const U32 *uni;
    unsigned rows;
    unsigned depth;
};

__global__ void normalize_rows(const U32 *starts, U32 *counts, unsigned rows) {
    unsigned row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= rows) return;
    unsigned begin = starts[row], end = starts[row+1];
    U64 total = 0;
    for (unsigned j = begin; j < end; ++j) total += counts[j];
    U64 cumulative = 0, before = 0;
    for (unsigned j = begin; j < end; ++j) {
        cumulative += counts[j];
        U64 after = Q * cumulative / total;
        counts[j] = static_cast<U32>(after - before);
        before = after;
    }
}

__global__ void normalize_floor(U32 *uni) {
    U64 total = 0;
    for (unsigned t = 1; t < V; ++t) total += U64(uni[t]) + 1;
    U64 cumulative = 0, before = 0;
    for (unsigned t = 1; t < V; ++t) {
        cumulative += U64(uni[t]) + 1;
        U64 after = (Q - (V - 1)) * cumulative / total;
        uni[t] = 1 + static_cast<U32>(after - before);
        before = after;
    }
}

__device__ unsigned find_row(Index index, U64 key) {
    unsigned lo = 0, hi = index.rows;
    while (lo < hi) {
        unsigned mid = lo + (hi - lo) / 2;
        if (index.keys[mid] < key) lo = mid + 1;
        else hi = mid;
    }
    return lo < index.rows && index.keys[lo] == key ? lo : NO_ROW;
}

__device__ unsigned resolve(Index index, const ID *ids, size_t i, unsigned *active) {
    U64 digits = 0;
    unsigned fallback = NO_ROW, has_row = 0;
    bool can_extend = true;
    for (unsigned slot = 0; slot < index.depth; ++slot) {
        if (can_extend && i > slot) {
            unsigned id = ids[i - slot - 1];
            if (id != 1) {
                digits |= U64(id) << (36 - slot * 12);
                unsigned exact = find_row(index, (U64(slot + 1) << 48) | digits);
                if (exact != NO_ROW) {
                    fallback = exact;
                    has_row |= 1U << slot;
                }
            }
            can_extend = id > 1;
        }
        active[slot] = fallback;
    }
    return has_row;
}

__device__ U32 expert(Index index, unsigned row, ID target) {
    if (row == NO_ROW) return index.uni[target];
    unsigned lo = index.starts[row], end = index.starts[row+1], hi = end;
    while (lo < hi) {
        unsigned mid = lo + (hi - lo) / 2;
        if (index.followers[mid] < target) lo = mid + 1;
        else hi = mid;
    }
    return lo < end && index.followers[lo] == target ? index.table[lo] : 0;
}

__device__ U64 mixed(Index index, const U32 *weights, const unsigned *active, ID target) {
    U64 mass = U64(weights[0]) * index.uni[target];
    for (unsigned k = 0; k < index.depth; ++k) {
        mass += U64(weights[k+1]) * expert(index, active[k], target);
    }
    return mass;
}

__global__ void target_mass(Index index, const ID *tokens, size_t n, const U32 *weights,
                            U64 *masses, uint8_t *hits) {
    size_t i = size_t(blockIdx.x) * blockDim.x + threadIdx.x;
    if (i >= n) return;
    ID target = tokens[i];
    if (!target) {
        masses[i] = 0;
        hits[i] = 0;
        return;
    }
    unsigned active[DEPTH];
    hits[i] = static_cast<uint8_t>(resolve(index, tokens, i, active));
    masses[i] = mixed(index, weights, active, target);
}

__global__ void fit_experts(Index index, const ID *tokens, size_t n, U32 *experts) {
    size_t i = size_t(blockIdx.x) * blockDim.x + threadIdx.x;
    if (i >= n) return;
    ID target = tokens[i];
    unsigned active[DEPTH];
    if (target) resolve(index, tokens, i, active);
    experts[i] = index.uni[target];
    for (unsigned k = 0; k < index.depth; ++k) {
        experts[n * (k + 1) + i] = target ? expert(index, active[k], target) : 0;
    }
}

__global__ void responsibilities(const U32 *experts, size_t n, unsigned depth,
                                 const U32 *weights, U64 *totals) {
    __shared__ U64 block_sums[DEPTH];
    if (threadIdx.x < depth) block_sums[threadIdx.x] = 0;
    __syncthreads();
    size_t i = size_t(blockIdx.x) * blockDim.x + threadIdx.x;
    if (i < n && experts[i]) {
        U64 weighted[DEPTH];
        U64 mass = U64(weights[0]) * experts[i];
        for (unsigned k = 0; k < depth; ++k) {
            weighted[k] = U64(weights[k + 1]) * experts[n * (k + 1) + i];
            mass += weighted[k];
        }
        for (unsigned k = 0; k < depth; ++k) {
            atomicAdd(block_sums + k, ((weighted[k] << 16) + mass / 2) / mass);
        }
    }
    __syncthreads();
    if (threadIdx.x < depth) atomicAdd(totals + threadIdx.x, block_sums[threadIdx.x]);
}

__global__ void update_weights(unsigned depth, const U64 *totals, U32 *weights) {
    U64 total = 0, cumulative = 0, before = 0;
    for (unsigned k = 0; k < depth; ++k) total += totals[k];
    // All conditional responsibilities can be zero on a degenerate tiny fixture.
    if (!total) return;
    for (unsigned k = 0; k < depth; ++k) {
        cumulative += totals[k];
        U64 after = (256 - FLOOR - depth) * cumulative / total;
        weights[k+1] = 1 + static_cast<U32>(after - before);
        before = after;
    }
}

__global__ void greedy_witness(Index index, const ID *tokens, size_t n, const U32 *weights,
                               size_t stride, unsigned witnesses, U64 *records) {
    unsigned j = blockIdx.x * blockDim.x + threadIdx.x;
    if (j >= witnesses) return;
    size_t i = size_t(j) * stride;
    U64 *out = records + j * (DEPTH + 2);
    if (i >= n || !tokens[i]) {
        for (unsigned k = 0; k < DEPTH + 2; ++k) out[k] = 0;
        return;
    }
    unsigned active[DEPTH];
    resolve(index, tokens, i, active);
    U64 best = 0;
    ID choice = 1;
    // Exhaustive over the output alphabet. Rust serves through the sparse candidate union.
    for (ID target = 1; target < V; ++target) {
        U64 mass = mixed(index, weights, active, target);
        if (mass > best) {
            choice = target;
            best = mass;
        }
    }
    out[0] = choice;
    out[1] = best;
    for (unsigned k = 0; k < DEPTH; ++k) {
        out[k+2] = k < index.depth && active[k] != NO_ROW ? index.keys[active[k]] : 0;
    }
}

template<class T> T word(const uint8_t *raw) {
    T value = 0;
    for (unsigned i = 0; i < sizeof(T); ++i) value |= T(raw[i]) << (i * 8);
    return value;
}

int main(int argc, char **argv) try {
    if (argc != 6) throw std::runtime_error("expected fit|score STATE IDS OUTPUT STRIDE");
    std::string mode = argv[1];
    if (mode != "fit" && mode != "score") throw std::runtime_error("unknown context-score mode");
    auto words = read<uint8_t>(argv[2]);
    auto ids = read<ID>(argv[3]);
    size_t stride = std::stoul(argv[5]);
    if (!stride || words.size() <= HEADER || (words.size() - HEADER) % ROW_BYTES ||
        ids.front() != 0 || ids.back() != 1)
        throw std::runtime_error("invalid context-score shape");
    std::vector<U32> raw_weights(COMPONENTS), raw_uni(V);
    unsigned depth = 0;
    for (unsigned k = 0; k < COMPONENTS; ++k) {
        raw_weights[k] = word<U32>(words.data() + 4 * k);
        if (raw_weights[k]) depth = k;
    }
    if (!depth || raw_weights[0] != FLOOR) throw std::runtime_error("bad context-score weights");
    for (unsigned t = 0; t < V; ++t)
        raw_uni[t] = word<U32>(words.data() + (COMPONENTS + t) * 4);
    std::vector<U64> keys;
    std::vector<U32> starts, counts;
    std::vector<ID> followers;
    for (size_t pos = HEADER; pos < words.size(); pos += ROW_BYTES) {
        U64 value = word<U64>(words.data() + pos), key = value >> 12;
        if (keys.empty() || keys.back() != key) {
            keys.push_back(key);
            starts.push_back(counts.size());
        }
        counts.push_back(word<U32>(words.data() + pos + 8));
        followers.push_back(static_cast<ID>(value & 0xfff));
    }
    starts.push_back(counts.size());
    Device<U64> d_keys(keys.size());
    Device<U32> d_starts(starts.size()), d_counts(counts.size()), d_uni(V), d_weights(COMPONENTS);
    Device<ID> d_followers(followers.size()), tokens(ids.size());
    upload(d_keys.p, keys);
    upload(d_starts.p, starts);
    upload(d_counts.p, counts);
    upload(d_uni.p, raw_uni);
    upload(d_weights.p, raw_weights);
    upload(d_followers.p, followers);
    upload(tokens.p, ids);
    Index index{d_keys.p, d_starts.p, d_followers.p, d_counts.p, d_uni.p,
                static_cast<unsigned>(keys.size()), depth};
    normalize_rows<<<(index.rows+255)/256,256>>>(d_starts.p,d_counts.p,index.rows);
    normalize_floor<<<1,1>>>(d_uni.p);
    auto blocks = static_cast<unsigned>((ids.size() + 255) / 256);
    std::ofstream out(argv[4], std::ios::binary);
    if (mode == "fit") {
        Device<U32> experts(ids.size() * (depth + 1));
        Device<U64> totals(depth);
        fit_experts<<<blocks,256>>>(index, tokens.p, ids.size(), experts.p);
        for (unsigned pass = 0; pass < 4; ++pass) {
            check(cudaMemset(totals.p, 0, depth * sizeof(U64)));
            responsibilities<<<blocks,256>>>(experts.p, ids.size(), depth, d_weights.p, totals.p);
            update_weights<<<1,1>>>(depth, totals.p, d_weights.p);
            save(out, COMPONENTS, d_weights.p);
        }
    } else {
        Device<U64> masses(ids.size());
        Device<U32> bits(ids.size());
        Device<uint8_t> hits(ids.size());
        unsigned probes = (ids.size() + stride - 1) / stride;
        Device<U64> witnesses(probes * (DEPTH + 2));
        target_mass<<<blocks,256>>>(index, tokens.p, ids.size(), d_weights.p, masses.p, hits.p);
        account<<<blocks,256>>>(masses.p, ids.size(), bits.p);
        greedy_witness<<<(probes+127)/128,128>>>(index, tokens.p, ids.size(), d_weights.p, stride,
                                                 probes, witnesses.p);
        save(out, ids.size(), masses.p);
        save(out, ids.size(), bits.p);
        save(out, ids.size(), hits.p);
        save(out, size_t(probes) * (DEPTH + 2), witnesses.p);
    }
    check(cudaDeviceSynchronize());
    return 0;
} catch (const std::exception &error) {
    std::fprintf(stderr, "%s\n", error.what());
    return 1;
}
