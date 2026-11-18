// Fixed-width finite E8 occurrence selection. No labels enter the selection
// kernel. Training totals exact source hits at the 192 legal native settings.
#define D11_CONST __device__ __constant__
#include "d11_tables.h"
typedef unsigned int u32;

extern "C" __global__ void occurrence_positions(const u32 *ids, const u32 *lengths,
        const u32 *roots, u32 rows, u32 start_rule, u32 rule_count, u32 *positions) {
    u32 i=blockIdx.x*blockDim.x+threadIdx.x;
    if (i>=rows*rule_count) return;
    u32 row=i/rule_count, rule=start_rule+i%rule_count;
    u32 q=rule/6u, d=1u+rule%6u, n=lengths[row];
    u32 winner=0xffffffffu, best=0xffffffffu;
    if (q<n) {
        u32 a=n-1u-q, base=192u*row, query=roots[ids[base+a]];
        for (u32 p=d;p<a;++p) {
            u32 code=roots[ids[base+p-d]], loss=0;
            for (u32 b=0;b<4;++b){
                u32 x=(code>>(4*b))&15u, y=(query>>(4*b))&15u;
                for (u32 c=0;c<8;++c){
                    int diff=(int)D11_ROOTS[x][c]-(int)D11_ROOTS[y][c];
                    loss+=(u32)(diff<0?-diff:diff);
                }
            }
            if(loss<best){best=loss;winner=p;}
        }
    }
    positions[i]=winner;
}

extern "C" __global__ void occurrence_fit(const u32 *positions,const u32 *labels,
        u32 rows,u32 *counts){
    u32 i=blockIdx.x*blockDim.x+threadIdx.x;
    if(i>=2u*192u)return;
    u32 arm=i/192u, rule=i%192u, hits=0;
    for(u32 j=0;j<rows;++j)hits+=positions[j*192u+rule]==labels[j*2u+arm];
    counts[i]=hits;
}

extern "C" __global__ void occurrence_winners(const u32 *counts,u32 *winners){
    u32 arm=threadIdx.x;
    if(arm>=2)return;
    u32 best=0;
    for(u32 i=1;i<192u;++i)if(counts[arm*192u+i]>counts[arm*192u+best])best=i;
    winners[arm]=best;
}
