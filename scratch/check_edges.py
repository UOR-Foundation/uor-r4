import struct

def check_r4g1():
    with open("r4g1_compiled/compiled.r4g1", "rb") as f:
        data = f.read()

    print(f"Total file size: {len(data)} bytes")
    
    magic = data[0:4]
    print(f"Magic: {magic}")
    major = data[4]
    minor = data[5]
    endianness = data[6]
    alignment_log2 = data[7]
    print(f"Version: {major}.{minor}, endianness: {endianness}, align_log2: {alignment_log2}")
    
    total_len = struct.unpack_from("<Q", data, 8)[0]
    print(f"Total len: {total_len}")
    
    section_count = struct.unpack_from("<I", data, 16)[0]
    flags = struct.unpack_from("<I", data, 20)[0]
    print(f"Section count: {section_count}, flags: {flags}")
    
    HEADER_LEN = 88
    SECTION_ENTRY_LEN = 16
    
    # Correct section ID mapping from types.rs
    SECTION_NAMES = {
        0x01: "HEAD", 0x02: "CODE", 0x03: "NODE", 0x04: "EDGE",
        0x05: "ROUT", 0x06: "EMIT", 0x07: "EXCT", 0x08: "PROV",
    }
    
    sections = {}
    for i in range(section_count):
        base = HEADER_LEN + i * SECTION_ENTRY_LEN
        sid = struct.unpack_from("<I", data, base)[0]
        sflags = struct.unpack_from("<I", data, base + 4)[0]
        soffset = struct.unpack_from("<I", data, base + 8)[0]
        slength = struct.unpack_from("<I", data, base + 12)[0]
        
        name = SECTION_NAMES.get(sid, f"UNKNOWN({sid})")
        print(f"  Section {name} (id=0x{sid:02X}): offset={soffset}, length={slength}, flags={sflags}")
        sections[sid] = (soffset, slength)
    
    # Parse HEAD section (id=0x01)
    node_count_head = 0
    edge_count_head = 0
    if 0x01 in sections:
        hoff, hlen = sections[0x01]
        head_data = data[hoff:hoff+hlen]
        print(f"\n=== HEAD section: {hlen} bytes ===")
        # HEAD layout (from induction.rs):
        # 0..32: teacher_cid
        # 32..64: tokenizer_cid  
        # 64..96: corpus_construction_cid
        # 96..128: corpus_certification_cid
        # 128..148: hf_revision
        # 148..180: compiler_version_hash
        # 180..184: A (max_frontier_width) u32
        # 184..188: C (MAX_CANDIDATES) u32
        # 188..190: W (SIG_WORDS) u16
        # 190..194: K (SHORTLIST_SIZE) u32
        # 194..198: E (max_emission_entries) u32
        # 198..202: D (MAX_PROGRAM_STEPS) u32
        # 202..206: node_count u32
        # 206..210: edge_count u32
        # 210: depth_count u8
        # 211..216: fallback_policy (5 bytes)
        # 216..218: reserved (2 bytes)
        # 218..220: signature_bytes u16
        # 220..222: min_runtime_major u16
        # 222..224: min_runtime_minor u16
        # Wait... that's only 224 bytes but there should be more fields
        
        # Let me just read what the head module parses
        A = struct.unpack_from("<I", head_data, 180)[0]
        C = struct.unpack_from("<I", head_data, 184)[0]
        W = struct.unpack_from("<H", head_data, 188)[0]
        K = struct.unpack_from("<I", head_data, 190)[0]
        E = struct.unpack_from("<I", head_data, 194)[0]
        D = struct.unpack_from("<I", head_data, 198)[0]
        node_count_head = struct.unpack_from("<I", head_data, 202)[0]
        edge_count_head = struct.unpack_from("<I", head_data, 206)[0]
        depth_count = head_data[210]
        sig_bytes = struct.unpack_from("<H", head_data, 218)[0] if hlen > 218 else 0
        
        print(f"  A={A}, C={C}, W={W}, K={K}, E={E}, D={D}")
        print(f"  node_count={node_count_head}, edge_count={edge_count_head}, depth_count={depth_count}")
        print(f"  signature_bytes={sig_bytes}")
    
    # Parse NODE section (id=0x03, 30 bytes per record)
    if 0x03 in sections:
        noff, nlen = sections[0x03]
        node_data = data[noff:noff+nlen]
        num_nodes = nlen // 30
        print(f"\n=== NODE section: {nlen} bytes, {num_nodes} nodes (HEAD says {node_count_head}) ===")
        for ni in range(min(num_nodes, 20)):
            base = ni * 30
            child_start = struct.unpack_from("<I", node_data, base)[0]
            child_len = struct.unpack_from("<H", node_data, base + 4)[0]
            fwd_start = struct.unpack_from("<I", node_data, base + 6)[0]
            fwd_len = struct.unpack_from("<H", node_data, base + 10)[0]
            emit_start = struct.unpack_from("<I", node_data, base + 12)[0]
            emit_len = struct.unpack_from("<H", node_data, base + 16)[0]
            proto_start = struct.unpack_from("<I", node_data, base + 18)[0]
            mask_start = struct.unpack_from("<I", node_data, base + 22)[0]
            radius = struct.unpack_from("<H", node_data, base + 26)[0]
            depth = node_data[base + 28]
            nflags = node_data[base + 29]
            print(f"  Node {ni}: depth={depth} emit_start={emit_start} emit_len={emit_len} "
                  f"child_start={child_start} child_len={child_len} "
                  f"fwd_start={fwd_start} fwd_len={fwd_len} radius={radius} "
                  f"proto_word={proto_start} mask_word={mask_start}")
    
    # Parse EDGE section (id=0x04)
    if 0x04 in sections:
        eoff, elen = sections[0x04]
        edge_data = data[eoff:eoff+elen]
        
        # Edges are 16 bytes each, followed by reverse index (4 bytes each)
        # Total EDGE section = edge_count * 16 + edge_count * 4 = edge_count * 20
        # So edge_count = elen / 20 if we don't have HEAD
        if edge_count_head == 0:
            edge_count_head = elen // 20
        
        print(f"\n=== EDGE section: {elen} bytes, {edge_count_head} edges ===")
        kind_counts = {}
        for ei in range(edge_count_head):
            base = ei * 16
            if base + 16 > elen:
                break
            src = struct.unpack_from("<I", edge_data, base)[0]
            dst = struct.unpack_from("<I", edge_data, base + 4)[0]
            score_q = struct.unpack_from("<i", edge_data, base + 8)[0]
            kind = edge_data[base + 12]
            eflags = edge_data[base + 13]
            kind_counts[kind] = kind_counts.get(kind, 0) + 1
            if ei < 20:
                EDGE_KINDS = {0: "Refine", 1: "Neighbor", 2: "Transition"}
                print(f"  Edge {ei}: src={src} -> dst={dst}, kind={kind}({EDGE_KINDS.get(kind, '?')}), score_q={score_q}")
        
        print(f"\n  Edge kind totals: {kind_counts}")
        for k, v in sorted(kind_counts.items()):
            EDGE_KINDS = {0: "Refine", 1: "Neighbor", 2: "Transition"}
            print(f"    Kind {k} ({EDGE_KINDS.get(k, '?')}): {v}")
    
    # Parse EMIT section (id=0x06)
    if 0x06 in sections:
        emoff, emlen = sections[0x06]
        emit_data = data[emoff:emoff+emlen]
        print(f"\n=== EMIT section: {emlen} bytes ===")
        # First 4 bytes: StorageDescriptor
        width = emit_data[0]
        shift = struct.unpack_from("b", emit_data, 1)[0]
        zero_point = struct.unpack_from("<h", emit_data, 2)[0]
        print(f"  StorageDescriptor: width={width}, shift={shift}, zero_point={zero_point}")
        
        # Show first emission entries (after descriptor at byte 4)
        print(f"  First emission entries (at byte offset 4):")
        for i in range(min(10, (emlen - 4) // 8)):
            base = 4 + i * 8
            token = struct.unpack_from("<i", emit_data, base)[0]
            count = struct.unpack_from("<i", emit_data, base + 4)[0]
            print(f"    Entry {i}: token={token}, count={count}")
        
        # Check Node 0's emission pointer
        if 0x03 in sections:
            noff, nlen = sections[0x03]
            node0_data = data[noff:noff+30]
            emit_start_0 = struct.unpack_from("<I", node0_data, 12)[0]
            emit_len_0 = struct.unpack_from("<H", node0_data, 16)[0]
            print(f"\n  Node 0 emission: start={emit_start_0}, len={emit_len_0}")
            if emit_start_0 == 0:
                print(f"  *** BUG: Node 0 emission_start=0 reads the StorageDescriptor as data! ***")
                print(f"      It would interpret descriptor bytes as: token={struct.unpack_from('<I', emit_data, 0)[0]}")
            
            # Show what Node 0 would actually read
            print(f"  What Node 0 reads (start={emit_start_0}, {emit_len_0} entries of 8 bytes):")
            for i in range(min(emit_len_0, 10)):
                base = emit_start_0 + i * 8
                if base + 8 <= emlen:
                    token = struct.unpack_from("<i", emit_data, base)[0]
                    count = struct.unpack_from("<i", emit_data, base + 4)[0]
                    print(f"    [{i}] token={token} count={count}")
                    
            # Check a few region nodes too
            num_nodes = nlen // 30
            for ni in range(1, min(num_nodes, 5)):
                node_data_i = data[noff + ni*30:noff + (ni+1)*30]
                emit_start_i = struct.unpack_from("<I", node_data_i, 12)[0]
                emit_len_i = struct.unpack_from("<H", node_data_i, 16)[0]
                print(f"\n  Node {ni} emission: start={emit_start_i}, len={emit_len_i}")
                for j in range(min(emit_len_i, 5)):
                    base = emit_start_i + j * 8
                    if base + 8 <= emlen:
                        token = struct.unpack_from("<i", emit_data, base)[0]
                        count = struct.unpack_from("<i", emit_data, base + 4)[0]
                        print(f"    [{j}] token={token} count={count}")

check_r4g1()
