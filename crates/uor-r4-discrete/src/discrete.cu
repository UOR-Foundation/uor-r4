// Integer-only SpiralCore/FBS path. Geometry arrays are compiled from the
// source-bound signed-axis action graph; no learned coefficient grid exists.
// Multiplication below is storage indexing only. Model values use gather,
// sign, add/subtract, comparison and finite action-table lookup.
typedef unsigned int U;
typedef unsigned long long W;
#define ROOT 0
#define AXIS 128
#define COMPOSE 640
#define BFS 4736
#define BRANCH 4992
#define GENERATOR 5040
#define INVERSE 5055
#define IDENTITY 5119
#define ABSENT 0xffffffffu
#define MAX_WIDTH 128

__device__ int magnitude(int x) { return x < 0 ? -x : x; }
__device__ U compose(const U* g,U a,U b) { return g[COMPOSE+a*64+b]; }
__device__ void action(const U* g,U a,const int* x,int* y) {
    for(U d=0;d<8;++d) { U axis=g[AXIS+a*8+d]; int v=x[axis&7]; y[d]=(axis&8)?-v:v; }
}
__device__ U move_action(const U* g,U candidate) { return candidate ? g[GENERATOR+candidate-1] : g[IDENTITY]; }
__device__ void table_block(const int* table,const U* g,U token,U block,U blocks,U kind,U index,U candidate,int* out) {
    const int* p=table+(token*blocks+block)*8;
    if(kind==1 && index==token*blocks+block && candidate) action(g,move_action(g,candidate),p,out);
    else for(U d=0;d<8;++d) out[d]=p[d];
}
__device__ void input_block(const int* input,const U* ids,const U* g,U row,U pos,U block,const U* s,U kind,U index,U candidate,int* out) {
    U at=row*s[2]+pos;
    const int* p=input+(at*s[1]+block)*8;
    if(kind==1 && index==ids[at]*s[1]+block && candidate) action(g,move_action(g,candidate),p,out);
    else for(U d=0;d<8;++d) out[d]=p[d];
}
// Coordinate displacement is primary. Coordinate Hamming breaks equal
// displacement; retained port identity resolves a remaining tie.
__device__ void root_cost(const int* x,const U* g,U root,U* distance,U* hamming) {
    *distance=0;*hamming=0;
    for(U d=0;d<8;++d) { int y=(int)g[ROOT+root*8+d]; *distance+=(U)magnitude(x[d]-y); *hamming+=(x[d]!=y); }
}
__device__ bool route_vector(const int* input,const U* ids,const U* routes,const U* plan,const U* g,const U* s,U row,U kind,U index,U candidate,int* output) {
    for(U block=0;block<s[1];++block) {
        int query[8];input_block(input,ids,g,row,s[2]-1,block,s,kind,index,candidate,query);
        U node=ABSENT,root=0,total=g[IDENTITY];
        for(U stage=0;stage<s[5];++stage) {
            U parameter=block*s[5]+stage;
            U frame=routes[parameter];
            if(frame>=64) return false;
            if(kind==0 && index==parameter && candidate) frame=compose(g,move_action(g,candidate),frame);
            int q[8];action(g,frame,query,q);
            U count=stage==0?s[7]:3;
            U best=ABSENT,best_distance=ABSENT,best_hamming=ABSENT,goal=0;
            for(U port=0;port<count;++port) {
                U r=stage==0?port:g[BRANCH+root*3+port];
                U distance,hamming;root_cost(q,g,r,&distance,&hamming);
                if(distance<best_distance || (distance==best_distance && hamming<best_hamming)) {
                    best=port;best_distance=distance;best_hamming=hamming;goal=r;
                }
            }
            if(best==ABSENT) return false;
            total=compose(g,g[BFS+root*16+goal],total);root=goal;
            node=stage==0?plan[s[6]*4+best]:plan[node*4+best];
            if(node>=s[6]) return false;
            U atom=plan[node*4+3];
            if(atom!=ABSENT) {
                if(atom>=s[2]) return false;
                int carrier[8];input_block(input,ids,g,row,atom,block,s,kind,index,candidate,carrier);
                action(g,total,carrier,output+block*8);break;
            }
            if(stage+1==s[5]) return false;
        }
    }
    return true;
}
__device__ U address_distance(const int* q,const U* g,const U* s,U cell) {
    U a,ha,b,hb;root_cost(q,g,cell/16,&a,&ha);root_cost(q+(s[1]-1)*8,g,cell%16,&b,&hb);return a+b;
}
__device__ U token_distance(const int* q,const int* table,const U* g,const U* s,U token,U kind,U index,U candidate) {
    U sum=0;
    for(U block=0;block<s[1];++block) { int v[8];table_block(table,g,token,block,s[1],kind,index,candidate,v);for(U d=0;d<8;++d) sum+=(U)magnitude(q[block*8+d]-v[d]); }
    return sum;
}
__device__ W repeated_add(W value,U times) { W total=0;for(U i=0;i<times;++i)total+=value;return total; }
// Numerator over common denominator 8*width. Target is scored even when
// geometric admission omits its address. No target participates in admission.
__device__ W writer(const int* q,const int* table,const U* g,const U* s,U target,U kind,U index,U candidate,U* prediction) {
    bool used[256];for(U i=0;i<256;++i)used[i]=false;
    U best_token=ABSENT,best_distance=ABSENT,rival_distance=ABSENT,rival_address=ABSENT;
    U target_cell=target%256;
    for(U k=0;k<s[3];++k) {
        U cell=ABSENT,cost=ABSENT;
        for(U c=0;c<256 && c<s[0];++c) if(!used[c]) { U d=address_distance(q,g,s,c);if(d<cost){cell=c;cost=d;} }
        if(cell==ABSENT)break;
        used[cell]=true;
        if(cell!=target_cell && cost<rival_address)rival_address=cost;
        for(U token=cell;token<s[0];token+=256) {
            U d=token_distance(q,table,g,s,token,kind,index,candidate);
            if(d<best_distance || (d==best_distance && token<best_token)){best_token=token;best_distance=d;}
            if(token!=target && d<rival_distance)rival_distance=d;
        }
    }
    *prediction=best_token;
    if(rival_distance==ABSENT)return ~(W)0;
    U width=s[1]*8;
    U td=token_distance(q,table,g,s,target,kind,index,candidate);
    U ad=address_distance(q,g,s,target_cell);
    W tm=width+td<=rival_distance ? 0 : width+td-rival_distance;
    W am=rival_address==ABSENT || 8+ad<=rival_address ? 0 : 8+ad-rival_address;
    return (tm<<3)+repeated_add(am,width);
}
extern "C" __global__ void discrete_init(int* table,U* routes,const U* g,const U* s,U seed,U* status) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;
    U elements=s[0]*s[1]*8;
    if(i<elements) {
        U block=i/8;U state=seed^block^0x9e3779b9u;
        state^=state<<13;state^=state>>17;state^=state<<5;
        table[i]=(int)g[ROOT+(state&15)*8+(i&7)];
    }
    if(i<s[1]*s[5])routes[i]=g[IDENTITY];
    if(i==0)*status=0;
}
extern "C" __global__ void discrete_encode(const int* table,const U* ids,U count,U width,U vocabulary,int* out,U* status) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;if(i>=count*width)return;
    U token=ids[i/width];if(token>=vocabulary){atomicExch(status,1u);return;}out[i]=table[token*width+i%width];
}
extern "C" __global__ void discrete_forward(const int* table,const U* routes,const U* ids,const int* input,const U* targets,const U* plan,const U* g,const U* s,int* out,U* predictions,W* losses,U* status) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    int q[MAX_WIDTH];if(!route_vector(input,ids,routes,plan,g,s,row,2,0,0,q)){atomicExch(status,2u);return;}
    for(U d=0;d<s[1]*8;++d)out[row*s[1]*8+d]=q[d];
    losses[row]=writer(q,table,g,s,targets[row],2,0,0,predictions+row);
    if(losses[row]==~(W)0)atomicExch(status,8u);
}
extern "C" __global__ void discrete_counterfactual(const int* table,const U* routes,const U* ids,const int* input,const U* targets,const U* plan,const U* g,const U* s,U kind,U index,W* losses,U* status) {
    U candidate=blockIdx.x*blockDim.x+threadIdx.x;if(candidate>=16)return;
    W sum=0;
    for(U row=0;row<s[4];++row){int q[MAX_WIDTH];U prediction;
        if(!route_vector(input,ids,routes,plan,g,s,row,kind,index,candidate,q)){atomicExch(status,2u);return;}
        W cost=writer(q,table,g,s,targets[row],kind,index,candidate,&prediction);
        if(cost==~(W)0){sum=~(W)0;if(candidate==0)atomicExch(status,8u);break;}
        sum+=cost;
    }
    losses[candidate]=sum;
}
extern "C" __global__ void discrete_choose(const W* losses,const W* epoch,W* proposal) {
    if(blockIdx.x || threadIdx.x)return;U best=0;
    for(U i=1;i<16;++i)if(losses[i]<losses[best])best=i;
    proposal[0]=best;proposal[1]=losses[0];proposal[2]=losses[best];proposal[3]=*epoch;
}
extern "C" __global__ void discrete_apply(int* table,U* routes,const U* g,const U* s,const W* proposal,U kind,U index,W* epoch,U* status) {
    if(blockIdx.x || threadIdx.x)return;
    if(proposal[3]!=*epoch){*status=3;return;}
    if(proposal[0]>=16 || kind>1 || (kind==0 && index>=s[1]*s[5]) || (kind==1 && index>=s[0]*s[1])){*status=4;return;}
    if(proposal[0]==0)return;
    if(proposal[2]>=proposal[1] || *epoch==~(W)0){*status=5;return;}
    U a=move_action(g,(U)proposal[0]);
    if(kind==0)routes[index]=compose(g,a,routes[index]);
    else {int out[8];action(g,a,table+index*8,out);for(U d=0;d<8;++d)table[index*8+d]=out[d];}
    ++*epoch;
}
extern "C" __global__ void discrete_validate(const int* table,const U* routes,const U* g,const U* s,U* status) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;
    if(i<s[0]*s[1]) {
        bool found=false;
        for(U r=0;r<16;++r){bool equal=true;for(U d=0;d<8;++d)if(table[i*8+d]!=(int)g[r*8+d])equal=false;found|=equal;}
        if(!found)atomicExch(status,6u);
    }
    if(i<s[1]*s[5] && routes[i]>=64)atomicExch(status,7u);
}
