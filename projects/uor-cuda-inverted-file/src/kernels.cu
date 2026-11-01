typedef unsigned int U;
typedef long long D;
#define ABSENT 0xffffffffu
#define NO_DISTANCE 0x7fffffffffffffffLL

__device__ U root_id(const int* roots, const int* block) {
    for (U root = 0; root < 16; ++root) {
        bool same = true;
        for (U d = 0; d < 8; ++d) {
            if (block[d] != roots[root * 8 + d]) { same = false; break; }
        }
        if (same) return root;
    }
    return ABSENT;
}

__device__ U token_cell(const int* roots, const int* token) {
    const U first = root_id(roots, token);
    const U last = root_id(roots, token + 24);
    return first == ABSENT || last == ABSENT ? ABSENT : first * 16 + last;
}

__device__ D l1(const int* query, const int* token) {
    D distance = 0;
    for (U d = 0; d < 32; ++d) {
        const D delta = (D)query[d] - token[d];
        distance += delta < 0 ? -delta : delta;
    }
    return distance;
}

__device__ D cell_distance(const int* roots, const int* query, U cell) {
    D distance = 0;
    const int* first = roots + (cell / 16) * 8;
    const int* last = roots + (cell % 16) * 8;
    for (U d = 0; d < 8; ++d) {
        const D a = (D)query[d] - first[d];
        const D b = (D)query[24 + d] - last[d];
        distance += (a < 0 ? -a : a) + (b < 0 ? -b : b);
    }
    return distance;
}

__device__ void rank_bucket(
    const int* table, const int* query, const U* offsets, const U* members,
    U k, const U* admitted, U* top8
) {
    D distances[8];
    for (U i = 0; i < 8; ++i) { top8[i] = ABSENT; distances[i] = NO_DISTANCE; }
    for (U slot = 0; slot < k; ++slot) {
        const U cell = admitted[slot];
        if (cell == ABSENT) continue;
        for (U pos = offsets[cell]; pos < offsets[cell + 1]; ++pos) {
            const U token = members[pos];
            const D distance = l1(query, table + token * 32);
            for (U i = 0; i < 8; ++i) {
                if (distance < distances[i] || (distance == distances[i] && token < top8[i])) {
                    for (U j = 7; j > i; --j) {
                        distances[j] = distances[j - 1];
                        top8[j] = top8[j - 1];
                    }
                    distances[i] = distance;
                    top8[i] = token;
                    break;
                }
            }
        }
    }
}

__device__ void rank_full_scan(
    const int* roots, const int* table, U count, const int* query,
    U k, const U* admitted, U* top8
) {
    D distances[8];
    for (U i = 0; i < 8; ++i) { top8[i] = ABSENT; distances[i] = NO_DISTANCE; }
    for (U token = 0; token < count; ++token) {
        const U cell = token_cell(roots, table + token * 32);
        bool selected = false;
        for (U slot = 0; slot < k; ++slot) {
            if (admitted[slot] == cell) { selected = true; break; }
        }
        if (!selected) continue;
        const D distance = l1(query, table + token * 32);
        for (U i = 0; i < 8; ++i) {
            if (distance < distances[i] || (distance == distances[i] && token < top8[i])) {
                for (U j = 7; j > i; --j) {
                    distances[j] = distances[j - 1];
                    top8[j] = top8[j - 1];
                }
                distances[i] = distance;
                top8[i] = token;
                break;
            }
        }
    }
}

__device__ void admit_cells(
    const int* roots, const int* query, const U* offsets, U row, U k, U* admitted
) {
    D distances[16];
    for (U i = 0; i < 16; ++i) { admitted[row * 16 + i] = ABSENT; distances[i] = NO_DISTANCE; }
    for (U cell = 0; cell < 256; ++cell) {
        if (offsets[cell] == offsets[cell + 1]) continue;
        const D distance = cell_distance(roots, query, cell);
        for (U i = 0; i < k; ++i) {
            if (distance < distances[i] || (distance == distances[i] && cell < admitted[row * 16 + i])) {
                for (U j = k - 1; j > i; --j) {
                    distances[j] = distances[j - 1];
                    admitted[row * 16 + j] = admitted[row * 16 + j - 1];
                }
                distances[i] = distance;
                admitted[row * 16 + i] = cell;
                break;
            }
        }
    }
}

extern "C" __global__ void uor_if_build(
    const int* table, const int* roots, U count,
    U* offsets, U* members, U* cells, U* absent_count, U* status
) {
    if (blockIdx.x || threadIdx.x) return;
    if (count == 0) { *status = 1; return; }
    for (U token = 0; token < count; ++token) {
        const U cell = token_cell(roots, table + token * 32);
        if (cell == ABSENT) { *status = 2; return; }
        cells[token] = cell;
        ++offsets[cell + 1];
    }
    for (U cell = 0; cell < 256; ++cell) offsets[cell + 1] += offsets[cell];
    U cursor[256];
    for (U cell = 0; cell < 256; ++cell) cursor[cell] = offsets[cell];
    for (U token = 0; token < count; ++token) members[cursor[cells[token]]++] = token;
    *absent_count = 0;
    *status = 0;
}

extern "C" __global__ void uor_if_update(
    const int* table, const int* roots, U count, U moved_token,
    U* offsets, U* members, U* cells, U* absent_count, U* status
) {
    if (blockIdx.x || threadIdx.x) return;
    if (moved_token >= count) { *status = 1; return; }
    const U old_cell = cells[moved_token];
    const U new_cell = token_cell(roots, table + moved_token * 32);
    if (new_cell == ABSENT) { *status = 2; return; }
    if (old_cell == new_cell) { *status = 0; return; }
    if (old_cell != ABSENT && (old_cell > 255 || offsets[old_cell] > offsets[old_cell + 1])) {
        *status = 3;
        return;
    }
    U remove_pos = ABSENT;
    if (old_cell != ABSENT) {
        for (U pos = offsets[old_cell]; pos < offsets[old_cell + 1]; ++pos) {
            if (members[pos] == moved_token) { remove_pos = pos; break; }
        }
        if (remove_pos == ABSENT) { *status = 3; return; }
    } else if (*absent_count == 0) {
        *status = 3;
        return;
    }
    const U total_present = offsets[256];
    if (new_cell == ABSENT) { *status = 2; return; }
    if (old_cell == ABSENT) {
        U pos = offsets[new_cell];
        while (pos < offsets[new_cell + 1] && members[pos] < moved_token) ++pos;
        for (U at = total_present; at > pos; --at) members[at] = members[at - 1];
        members[pos] = moved_token;
        for (U cell = new_cell + 1; cell <= 256; ++cell) ++offsets[cell];
        --*absent_count;
    } else if (old_cell < new_cell) {
        U insert_before = offsets[new_cell];
        while (insert_before < offsets[new_cell + 1] && members[insert_before] < moved_token) ++insert_before;
        const U target = insert_before - 1;
        for (U pos = remove_pos; pos < target; ++pos) members[pos] = members[pos + 1];
        members[target] = moved_token;
        for (U cell = old_cell + 1; cell <= new_cell; ++cell) --offsets[cell];
    } else {
        U insert_pos = offsets[new_cell];
        while (insert_pos < offsets[new_cell + 1] && members[insert_pos] < moved_token) ++insert_pos;
        for (U pos = remove_pos; pos > insert_pos; --pos) members[pos] = members[pos - 1];
        members[insert_pos] = moved_token;
        for (U cell = new_cell + 1; cell <= old_cell; ++cell) ++offsets[cell];
    }
    cells[moved_token] = new_cell;
    *status = 0;
}

extern "C" __global__ void uor_if_probe(
    const int* table, const int* roots, const int* queries, U rows,
    const U* offsets, const U* members, const U* cells,
    U k, U* predictions, U* admitted, U* top8, U* status
) {
    const U row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= rows) return;
    if (k == 0 || k > 16) { status[row] = 1; return; }
    const int* query = queries + row * 32;
    admit_cells(roots, query, offsets, row, k, admitted);
    rank_bucket(table, query, offsets, members, k, admitted + row * 16, top8 + row * 8);
    predictions[row] = top8[row * 8];
    status[row] = predictions[row] == ABSENT ? 1 : 0;
}

extern "C" __global__ void uor_if_full_scan(
    const int* table, const int* roots, U count, const int* queries, U rows,
    const U* offsets, const U* members, const U* cells,
    U k, U* predictions, U* admitted, U* top8, U* status
) {
    const U row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= rows) return;
    if (k == 0 || k > 16) { status[row] = 1; return; }
    const int* query = queries + row * 32;
    admit_cells(roots, query, offsets, row, k, admitted);
    rank_full_scan(roots, table, count, query, k, admitted + row * 16, top8 + row * 8);
    predictions[row] = top8[row * 8];
    status[row] = predictions[row] == ABSENT ? 1 : 0;
}
