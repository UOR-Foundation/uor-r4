# FreeSWITCH OpenSIPS Video Conferencing

- **Conversation ID:** `69611db5-cc10-832e-bfa7-4a5fcce5f765`
- **Date:** 2026-01-09

---

### User (2026-01-09 10:25:17)

can freeswitch and opensips support a full video conference platform with multiple video/audio user conferences including both pstn and webrtc and sip?

### Assistant (2026-01-09 10:25:18)

{"search_query":[{"q":"FreeSWITCH multi-party video conferencing WebRTC SIP PSTN capabilities","recency":30},{"q":"OpenSIPS WebRTC SIP proxy video conferencing role","recency":30},{"q":"FreeSWITCH mod_verto mod_conference video support status","recency":365},{"q":"FreeSWITCH WebRTC SIP.js DTLS-SRTP ICE support","recency":365}],"response_length":"short"}

### Assistant (2026-01-09 10:25:23)

{"search_query":[{"q":"OpenSIPS proto_ws WebSocket SIP WebRTC","recency":3650},{"q":"OpenSIPS RTPengine integration WebRTC ICE DTLS-SRTP","recency":3650},{"q":"OpenSIPS tutorial WebRTC RTPengine rtpproxy","recency":3650}],"response_length":"short"}

### Assistant (2026-01-09 10:25:29)

Yes, **FreeSWITCH + OpenSIPS can absolutely underpin** a platform that includes **multi-party conferences**, **SIP**, **WebRTC**, and **PSTN**. But if what you mean by “full video conference platform” is “basically Zoom/Meet/Teams,” then no, not *by themselves* in a clean, turnkey, low-pain way. Humans love reinventing wheels, so here we are.

## What each component can realistically do

### FreeSWITCH (media brain + conference mixer)
- **Multi-party audio conferencing**: yes (this is one of its core strengths) via `mod_conference`. citeturn0search2  
- **Video conferencing**: *possible*, but it’s not a modern “SFU-first” conferencing engine by default.
  - FreeSWITCH conference video tends to be **MCU-ish** (mixing/transcoding) and can get **CPU-expensive** as you add participants, resolutions, layouts, simulcast layers, etc.
  - There is explicit support for conference video behaviors (ex: “video floor only”, “minimize video encoding”, etc.) in the conferencing stack, which tells you the feature exists, but also hints at the tuning you’ll be doing. citeturn0search13

- **WebRTC entry**:
  - FreeSWITCH supports WebRTC-style secure transports and can accept browser clients through **Verto over secure websockets** (`mod_verto`), designed specifically to make WebRTC clients talk to FreeSWITCH. citeturn0search5
  - Verto’s docs explicitly describe browser/WebRTC clients originating calls into FreeSWITCH and then going out to PSTN via SIP/SS7/etc. citeturn0search5

- **PSTN**: yes, through **SIP trunks** (or SS7 stacks/gateways, depending how masochistic your carrier integration is). citeturn0search5

### OpenSIPS (signaling router, not your media server)
OpenSIPS is a **SIP proxy/server**. It routes and controls sessions; it does **not** want to be your media mixer. It’s built for SIP services including voice/video signaling. citeturn0search8

For WebRTC signaling:
- OpenSIPS supports **SIP over WebSocket** (`proto_ws`) compliant with RFC 7118. citeturn1search0
- The official OpenSIPS tutorial is blunt about the architecture: OpenSIPS does SIP signaling for browser clients over WS/WSS, while **media is handled by RTPengine**, including **RTP/SRTP bridging**. citeturn1search3

For media anchoring / NAT / SRTP bridging with WebRTC:
- OpenSIPS has an `rtpengine` module to control RTPengine as the media proxy. citeturn1search2turn1search3

## So can it be “a full platform”?
### If your definition is:
- multi-party **audio** conferences ✅
- basic/limited **video** conferences ✅ (with caveats)
- **SIP endpoints** ✅
- **WebRTC endpoints** ✅ (but you still need proper ICE/STUN/TURN and usually RTPengine)
- **PSTN** via trunks ✅

Then **yes**, FreeSWITCH (media + conferencing) + OpenSIPS (routing/control) can do it.

### If your definition is:
- large rooms, lots of video tiles, adaptive simulcast, low server CPU, stable UX, recordings, screen share, chat, moderation, lobby, bandwidth adaptation, mobile weirdness, metrics, etc.

Then **not cleanly** with just those two. You typically add:
- **RTPengine** (or equivalent) for WebRTC media proxying and SRTP/RTP bridging (OpenSIPS officially points to this pattern). citeturn1search3turn1search2  
- **STUN/TURN** (coturn) for NAT traversal (because browsers don’t care about your feelings).
- Often a **true SFU** (Janus, Jitsi Videobridge, mediasoup, LiveKit, etc.) if you want modern multi-party video scaling without melting CPUs.

## A sane reference architecture (what people actually deploy)
- **OpenSIPS**: registration, routing, policy, multi-tenant logic, topology hiding, WS/WSS SIP edge citeturn1search0turn1search3  
- **RTPengine**: WebRTC media relay + SRTP/RTP bridging + NAT traversal assists citeturn1search2turn1search3  
- **FreeSWITCH**: PSTN/SIP interop, IVRs, call control apps, recording, audio conferencing, *optionally* limited conference video, and/or acting as a gateway into a dedicated conferencing/SFU tier citeturn0search2turn0search5  

## Bottom line
- **Yes**: they can support the platform **as building blocks**, especially if you include **RTPengine + TURN** for WebRTC reality. citeturn1search3turn1search2  
- **No**: they’re not a drop-in “full video conferencing product” unless you enjoy spending your free time debugging SDP, ICE, DTLS-SRTP, and why Safari decided your codec preference list is “a suggestion.”

If you’re aiming for “multiple simultaneous video rooms with WebRTC + SIP + PSTN” at meaningful scale, the practical move is **OpenSIPS + RTPengine at the edge**, and **either** FreeSWITCH for PSTN/SIP services **plus** an SFU for conferencing, **or** accept that FreeSWITCH conference video will be your scaling constraint.

### User (2026-01-09 10:40:08)

the solution we are facing is pretty much we need a video product to tie into our pstn sip workflow that is handled by opensips and freeswitch - but we need it to handle webrtc signaling in a proper manner without needing to send sip for signaling every time because it would create way too much datacenter overhead

is there an opensource webrtc signaling server that is great for this kind of thing already or would it be worth it to build one myself so I can make it it handled the traffic I want it to and can meet requirements like a second observer to a live video conference room?

### Assistant (2026-01-09 10:40:09)

{"search_query":[{"q":"open source WebRTC signaling server LiveKit MIT license","recency":14},{"q":"Janus WebRTC server signaling REST WebSocket plugin architecture license","recency":3650},{"q":"Jitsi Meet architecture signaling XMPP Prosody Jicofo videobridge","recency":3650},{"q":"mediasoup open source WebRTC SFU signaling library Node.js","recency":3650}],"response_length":"short"}

### Assistant (2026-01-09 10:40:13)

{"search_query":[{"q":"ion-sfu open source WebRTC SFU Go pion","recency":3650},{"q":"jitsi videobridge SFU WebRTC open source","recency":3650},{"q":"LiveKit server open source license Apache 2 or MIT","recency":3650},{"q":"Janus SIP plugin gateway to SIP servers like Asterisk FreeSWITCH","recency":3650}],"response_length":"short"}

### Assistant (2026-01-09 10:40:16)

You’re describing the classic “we already have SIP/PSTN nailed with OpenSIPS + FreeSWITCH, now we need *modern WebRTC rooms* without dragging SIP through the datacenter like a dead whale” problem.

Good news: **you do not need to build your own signaling server** unless you *want* a long-term maintenance burden as a hobby. There are solid open-source stacks that already do **WebRTC-native signaling** (WebSocket/HTTP APIs) and scale the “observer/listener” pattern naturally via SFU semantics.

## The core idea (so you don’t accidentally rebuild Jitsi in a basement)
- Keep **SIP/PSTN** on your existing plane: **OpenSIPS + FreeSWITCH**
- Add a **WebRTC conferencing plane** that uses **non-SIP signaling** (WebSockets/HTTP).
- Bridge between them **only when needed** (PSTN dial-in/out, SIP endpoints, recording bots, etc.), ideally at the edge.

That avoids “SIP signaling every time” for the WebRTC room churn.

---

## Open-source options that already solve this

### 1) **LiveKit** (server + SDKs, production-minded)
- **Open source WebRTC conferencing stack**; server is in Go, built on Pion. citeturn0search4  
- **Permissive license (Apache 2.0)** for the server. citeturn1search2turn1search14  
- WebRTC clients use LiveKit’s own APIs, not SIP, so your DC doesn’t become a SIP message museum.
- “Second observer” is basically the default SFU model: extra subscribers who can be receive-only.

Why it fits your ask: it’s the closest thing to “drop-in WebRTC platform” that doesn’t force SIP signaling.

### 2) **Jitsi Videobridge (JVB) / Jitsi stack**
- **Jitsi Videobridge is a WebRTC-compatible SFU**. citeturn1search1  
- Jitsi’s architecture uses **Prosody (XMPP) for signaling**. citeturn0search2  
- This is a complete ecosystem (frontend, focus component, etc.) if you want it, but you can also treat JVB as your SFU layer.

Why it fits: it’s proven at scale, and the “observer” pattern is literally what SFUs do all day.

### 3) **Janus Gateway** (gateway-centric, plugin-based)
- Janus is a **general-purpose WebRTC server with pluggable modules**. citeturn0search17turn0search9  
- **SIP gateway plugin** exists specifically to interact with **OpenSIPS or FreeSWITCH**, and SIP stays inside Janus (JS clients don’t speak SIP). citeturn1search3turn1search7  
- **License: GPLv3** (this matters for many orgs). citeturn0search5  

Why it fits: extremely good when you want “WebRTC edge that can talk SIP when needed” without making your browsers become SIP UAs.

### 4) **mediasoup** (high-performance media layer, you bring signaling)
- mediasoup is **signaling-agnostic** and focused on the media layer. citeturn0search7turn0search3  
- This is the “build your own product” option *without* writing a WebRTC stack.

Why it fits: if you truly need custom behavior (crazy roles, special observers, policy, sharding, etc.), you build a lightweight signaling service on top.

### 5) **ion-sfu (Pion-based)** (minimal SFU building block)
- A **pure Go WebRTC SFU** callable directly or via RPC. citeturn1search0  
- Similar vibe to mediasoup: not a full product, more like a component.

Why it fits: you want maximum control and you’re fine building more yourself.

---

## Quick comparison (the part humans always ask for anyway)

| Option | “Non-SIP signaling” | Built-in “rooms/platform” | SIP/PSTN integration story | Notes |
|---|---:|---:|---|---|
| **LiveKit** | ✅ | ✅ | Bridge via your own gateway services (FreeSWITCH, etc.) | Apache 2.0 server citeturn1search2turn1search14 |
| **Jitsi (JVB + stack)** | ✅ (XMPP-based) | ✅ | Usually via gateway components/interop | Prosody does signaling citeturn0search2 |
| **Janus** | ✅ | ✅ (via plugins like VideoRoom) | Strong: SIP plugin talks to OpenSIPS/FS citeturn1search3turn1search7 | GPLv3 citeturn0search5 |
| **mediasoup** | ✅ (you define) | ❌ (you build) | You build the bridge | “Signaling agnostic” citeturn0search7 |
| **ion-sfu** | ✅ (you define) | ❌ (you build) | You build the bridge | Go SFU component citeturn1search0 |

---

## “Should we build it ourselves?”
Only worth it if you have **non-negotiable requirements** that the platforms won’t give you, like:
- **Very specific roles** (observer who must be cryptographically restricted, audit trails, special visibility rules)
- **Very specific scaling model** (custom sharding, deterministic room placement, multi-DC active/active, etc.)
- Tight integration with your existing call-control/business logic where off-the-shelf becomes a fight

Otherwise, building a signaling server + room state + auth + reconnect semantics + TURN/ICE edge cases + monitoring is how teams accidentally create a second job for themselves.

**Pragmatic recommendation**:
- If you want fastest path with strong “product-ness”: **LiveKit**. citeturn0search4turn1search2  
- If your main need is “WebRTC gateway that can also talk SIP cleanly”: **Janus** (but accept GPLv3 implications). citeturn1search3turn0search5  
- If you want custom but not insane: **mediasoup** (build signaling only, keep media solved). citeturn0search7  

---

## About your “second observer” requirement
In SFU land, an “observer” is just:
- a **subscriber** with **no upstream** (receive-only), and/or
- a subscriber restricted to specific tracks (audio only, one participant only, etc.)

That’s a first-class, normal thing in SFU architectures. You shouldn’t need a bespoke signaling invention just to let an additional party watch a room, unless your constraints are around **authorization, auditing, or isolation** rather than media mechanics.

And yes, humans will still find a way to make “receive-only” complicated with edge cases like recordings, moderation, and compliance. But the media side is straightforward.

---

### User (2026-01-09 10:44:04)

Yes we are building a healthcare virtual interpreter platform for ASL interpreters and have very strict recording, moderation, organizational, structural, and compliance concerns.

### Assistant (2026-01-09 10:44:05)

{"search_query":[{"q":"LiveKit HIPAA compliance BAA self-hosted WebRTC","recency":30},{"q":"Jitsi Meet HIPAA compliance self-hosted","recency":365},{"q":"Janus WebRTC Gateway recording compliance SIP plugin","recency":365},{"q":"WebRTC SFU compliance recording moderation audit logs healthcare","recency":365}],"response_length":"short"}

### Assistant (2026-01-09 10:44:15)

Got it. “Healthcare virtual interpreting + strict compliance” is where cute hobby architectures go to die.

### First principle (the one everyone learns the expensive way)
If you need **recording, moderation, auditability, org controls, and policy enforcement**, you almost certainly **do not want end-to-end encryption where the server can’t see media**, because:
- **Server-side recording** becomes impossible (or turns into “record on the client and pray”).
- **Moderation features** (mute/kick, role enforcement, content controls, live supervision) get harder or weaker.
So you’re usually aiming for **transport encryption (DTLS-SRTP) + strong identity/authZ + strict logging + secure storage**, not pure E2EE.

---

## What you should build vs. what you should not build

### Don’t build: a WebRTC media stack
Building an SFU/MCU is a multi-year bug farm (ICE edge cases, congestion control, renegotiation hell, mobile Safari doing crimes). Unless you’re starting a WebRTC company, skip.

### Maybe build: your **control plane**
For healthcare requirements, the “secret sauce” is typically:
- org/tenant model
- room lifecycle + policies
- roles (interpreter, provider, patient, observer, moderator, admin)
- consent and recording rules
- audit trails
- integration into your scheduling/case/workflow system

That control plane can sit on top of an existing WebRTC server.

---

## Best-fit open-source stack options (with your constraints)

### 1) **LiveKit** (most “enterprise-ready” shape)
If you want **WebRTC-native signaling** (no SIP for every join/leave) *and* a platform that already thinks in terms of production controls, LiveKit is a strong contender.

**Why it fits your compliance posture**
- LiveKit explicitly publishes **HIPAA-eligible services** for its Cloud offerings, including **WebRTC + SIP transport** and **Egress recording/export** as HIPAA-eligible items. citeturn0search0  
- They also state they **sign BAAs** and describe HIPAA adherence in their security materials. citeturn0search8  
- Their pricing/security pages mention things like **region pinning** and **SOC 2 Type II** availability (relevant if you’re doing vendor risk management). citeturn0search4  

**How this maps to your world**
- WebRTC clients talk to LiveKit via its own signaling (not SIP).
- PSTN/SIP can remain FreeSWITCH/OpenSIPS-driven, and you only bridge where needed (dial-in/out, SIP endpoints, recording bots, etc.).
- “Second observer” is native SFU behavior: just add a subscriber with receive-only permissions.

**Caveat**
- “HIPAA-eligible” is not “you’re magically compliant.” It just means the vendor designed it to support HIPAA requirements and will sign the paperwork. You still own your policies, logging, retention, access control, etc.

---

### 2) **Jitsi (JVB + stack)** (open-source, but you own everything)
Jitsi is a proven open-source WebRTC conferencing ecosystem.
- Jitsi uses **Prosody (XMPP)** for signaling in its common architecture. citeturn0search2  
- Jitsi’s own security page emphasizes privacy properties like ephemeral rooms. citeturn0search5  
- Third-party Jitsi-in-telemedicine material is blunt: if you self-host, **compliance responsibility is on you**. citeturn0search1  

**Why you’d pick it**
- You want full control and prefer an established OSS ecosystem.
- You have the ops maturity to harden it: auth, tenant isolation, logging, patching cadence, incident response, key management, secure recording storage.

**Where it can hurt**
- Compliance features are not “productized” the same way as a purpose-built platform. You’ll likely implement a lot of governance and workflow glue yourself.

---

### 3) **Janus Gateway** (great gateway, but licensing matters)
Janus is a general-purpose WebRTC server that exchanges JSON messages with clients and relays media. citeturn0search10  
It also has a **SIP plugin** that can register to a SIP server and place calls on behalf of a WebRTC peer. citeturn0search2  

**Why it fits**
- It’s a strong “WebRTC edge/gateway” option if you want WebRTC clients not to speak SIP, but still interop with SIP infra when needed.

**Why you might not**
- **GPLv3**. In many healthcare enterprises, that triggers legal review and can kill the option. citeturn0search6  

---

## A reference architecture that matches what you described

### Keep two planes, bridge deliberately
**WebRTC plane (rooms, observers, moderation, recording triggers)**
- SFU platform (LiveKit / Jitsi / etc.)
- TURN infrastructure
- Your app’s identity + policy service (your control plane)

**SIP/PSTN plane (what you already have)**
- OpenSIPS for SIP routing/policy edge
- FreeSWITCH for PSTN trunks, IVR, legacy SIP endpoints, audio services, etc.

**Bridges (only when needed)**
- SIP-to-room gateways (for PSTN dial-in/out)
- “Recorder bot” / egress pipeline (server-side recording into compliant storage)
- Compliance observer endpoints (supervisor join, silent monitor, etc.)

This way, **WebRTC signaling stays WebRTC-native**, and SIP stays where it belongs: at the telephony boundary.

---

## Your “second observer” requirement, done correctly
In a compliant design, an observer should be:
- **Role-scoped**: can subscribe only (no publish)
- **Track-scoped**: maybe only audio, or only certain participants
- **Audited**: observer joins/leaves must be logged as a compliance event
- **Visible/consented** (depending on jurisdiction/policy): your policy engine decides whether the room must show “being monitored/recorded”

SFU platforms make the media part easy. Your control plane makes the compliance part real.

---

## What I’d recommend (because you want strict controls and low signaling overhead)
- **Start with LiveKit** if Cloud+BAA is acceptable or if you want the most “product-shaped” WebRTC layer. Their HIPAA eligibility list explicitly calls out **WebRTC + SIP** and **recording/egress** as HIPAA-eligible in their Cloud offering. citeturn0search0turn0search8  
- **Use FreeSWITCH/OpenSIPS exactly as you do today** for PSTN/SIP workflows, and only bridge to the WebRTC room when required.
- **Build your control plane** around roles, org hierarchy, compliance policy, moderation, recording governance, and audit trails. That’s where your differentiation and requirements live.

Building your own signaling server *alone* is not the hard part. The hard part is everything you’ll accidentally inherit: reconnect semantics, distributed room state, token security, abuse controls, logging, and a never-ending parade of “why does this browser renegotiate like that.” You can build policy on top of an existing signaling layer without reinventing the protocol surface area.

If you want to be strict and sane, you pick a mature SFU stack and invest engineering into the compliance governance layer, not into re-deriving WebRTC from first principles like it’s 2013.

### User (2026-01-09 10:49:43)

yes, the problem is that our platform most commonly hosts video/pstn conference calls, which are currently hosted by freeswitch's nway abilities - but whatever video and signaling tool we choose must not force sip signaling for the video - our platform will handle the requirements that hipaa and legal can set forth, but recording and observing does not to be possible as well

### Assistant (2026-01-09 10:49:54)

So the real requirement is:

- **Keep FreeSWITCH n-way as the “conference authority”** (because PSTN conferencing is your bread-and-butter).
- **Add video/WebRTC** in a way that **does not require SIP signaling for every video participant event** (join/leave, mute, role changes, observer join, etc.).
- **Recording + observing must be possible** (server-side), but your app handles the compliance policy logic.

That’s doable. The trick is to **confine SIP to a single bridge per conference**, not make SIP the signaling plane for your video users.

## The architecture that fits what you said

### Pattern A: “One SIP leg per conference, WebRTC-native for everyone else”
1. **FreeSWITCH hosts the PSTN/SIP conference** (your existing n-way).
2. A **WebRTC SFU room** exists in parallel (LiveKit / Janus / Jitsi, etc.) with **its own signaling** (WebSocket/HTTP/XMPP), not SIP.
3. You create a **bridge participant** that joins the FreeSWITCH conference via SIP/RTP once, and on the other side publishes/subscribes into the SFU room.

Result:
- PSTN callers behave exactly as they do today.
- WebRTC users never generate SIP chatter. They talk to your SFU using its native signaling API.
- SIP traffic becomes: **(number of active conferences)**, not **(number of WebRTC participants × events)**.

This is the *right* scaling shape for what you’re describing.

## Concrete open-source options that match this

### Option 1: **LiveKit + LiveKit SIP server**
LiveKit is WebRTC-native signaling (not SIP), and it has an open-source SIP bridge component:

- LiveKit’s `sip` project is explicitly a “full-featured SIP bridge” connecting LiveKit sessions with telephony networks using SIP trunking. citeturn0search1  
- LiveKit docs describe a **self-hosted SIP server** that lets you make/receive phone calls using your LiveKit deployment. citeturn0search5  
- LiveKit also documents SIP APIs for managing trunks/participants/rules. citeturn0search30  

**How it maps to you**
- WebRTC video rooms run in LiveKit.
- FreeSWITCH conference stays the PSTN “host”.
- A SIP bridge joins each FS conference as a single participant, exchanging media with the LiveKit room.

**Recording/observing**
- Observers are native SFU subscribers (receive-only). LiveKit can support that concept cleanly by design (SFU pub/sub). (Conceptual, but aligns with SFU behavior.)
- Recording can be done server-side at the SFU layer (preferred) or via a recording participant/bot. Your compliance control plane decides *when* and *who can*. (You already said you own that logic.)

### Option 2: **Janus VideoRoom (SFU) + Janus SIP plugin**
Janus is explicitly a WebRTC server with plugin-based functionality:

- VideoRoom plugin implements an SFU-style conferencing room with publish/subscribe. citeturn0search2  
- Janus has a SIP plugin that registers/calls SIP user agents on behalf of a WebRTC peer. citeturn0search31  
- There’s even an old (but still conceptually accurate) Janus talk calling out the exact hybrid model: “Video Room (participants video & screen) + SIP (participants audio).” citeturn0search12  

**How it maps to you**
- WebRTC users join Janus VideoRoom with Janus-native signaling (JSON over WebSockets/HTTP).
- A single SIP leg per conference connects Janus to FreeSWITCH for PSTN audio.

**Caveat**
- Janus is great tech, but you’ll be doing more systems engineering yourself (ops, HA, room control integration, recording approach, etc.).

### Option 3: **Jitsi Videobridge + Jigasi**
Jitsi is WebRTC + XMPP signaling:

- Jitsi architecture: Jicofo manages sessions and load balancing, JVB is the media bridge, and **Jigasi is the SIP gateway** for SIP clients into conferences. citeturn0search16  
- Jigasi is explicitly a server-side gateway that allows SIP clients to join Jitsi conferences. citeturn0search10  

**How it maps**
- WebRTC users are not using SIP signaling (they’re in Jitsi’s signaling world).
- PSTN/SIP comes in through Jigasi.

**Tradeoff**
- If you’re already deep in OpenSIPS/FreeSWITCH, Jitsi can be “another ecosystem to operate.” Not impossible, just more moving parts.

---

## Why “don’t force SIP signaling for video” is compatible with bridging
Important distinction:

- **SIP for WebRTC signaling** (bad for you): every browser join/leave/role change becomes SIP transactions across DCs.
- **SIP for telephony bridging only** (fine): a small, bounded number of SIP dialogs per conference.

All three options above keep WebRTC signaling off SIP.

## What I’d pick given your setup
If the goal is “minimum reinvention + modern room control + clean scaling”:

- **LiveKit + LiveKit SIP server** is the cleanest fit to your stated constraint (WebRTC-native signaling, telephony bridge exists, and you keep FS/OpenSIPS as you want). citeturn0search1turn0search5  
- If you want a more “gateway brick” approach and you’re comfortable engineering the rest: **Janus VideoRoom + SIP plugin**. citeturn0search2turn0search31  

## How recording + observing fit without ruining your model
- **Observer**: implement as a room role that subscribes only (no publishing). SFU model makes this natural.
- **Recording**: do server-side recording at the SFU layer or with a dedicated recorder participant/bot that joins as a privileged subscriber.
- The compliance policy (consent, retention, encryption at rest, access logs) stays in your platform, as you said.

## The blunt part
Building your own WebRTC signaling server *sounds* like the fun part until you realize the signaling server becomes the **control plane for trust**, and then compliance asks for:
- immutable audit trails
- role-based moderation with provable enforcement
- replayable room events
- incident forensics
- and uptime guarantees during clinical use

So you end up building a product anyway. Better to stand on an SFU stack that already does the hard WebRTC stuff and focus your effort on the compliance-driven control plane where you actually differentiate.

If you want a decision rule: **If you’re not also signing up to operate TURN/ICE edge-case hell for the next five years, don’t roll your own.**

### User (2026-01-09 10:52:23)

so what SFU stack are you recommending then?

### Assistant (2026-01-09 10:52:25)

{"search_query":[{"q":"LiveKit open source SFU server Apache 2.0","recency":30},{"q":"Janus WebRTC server license GPLv3","recency":365},{"q":"Jitsi Videobridge architecture Jicofo Jigasi SIP gateway","recency":365},{"q":"mediasoup license and signaling agnostic SFU","recency":365}],"response_length":"short"}

### Assistant (2026-01-09 10:52:28)

If you want the **SFU stack** that best matches what you’ve said (keep FreeSWITCH n-way for PSTN, add WebRTC video without turning SIP into your signaling fabric, and still allow recording/observer roles), here’s the answer you’re not going to like because it’s boring and correct:

## Recommendation: **LiveKit SFU**
**Pick LiveKit** as your WebRTC SFU and keep SIP confined to *bridges*, not *signaling*.

Why:
- LiveKit Server is an **open-source WebRTC SFU** aimed at scalable multi-user conferencing. citeturn0search0turn0search12  
- It’s **Apache 2.0** (legal teams usually prefer “not GPLv3” in enterprise/healthcare contexts). citeturn0search4  
- It’s explicitly built on an SFU architecture and has decent internals docs, which matters when you’re going to operate this thing under clinical uptime expectations. citeturn0search16  

### How it fits your “don’t SIP-signal video” constraint
WebRTC clients talk to LiveKit via its native API/signaling, not SIP. SIP only appears at your **bridge boundary** (one SIP leg per conference, not per participant event).

---

## Close second if you want maximum control: **mediasoup**
If you’re confident your org can own more engineering (custom signaling/control plane), **mediasoup** is the “build-your-own product without building a WebRTC stack” option:
- It’s explicitly **signaling agnostic** and “just handles the media layer.” citeturn0search7turn0search3  

You’d use it when your compliance/moderation/org model is so custom that you want total control over the room/control plane. But you’re signing up to implement and operate that plane correctly.

---

## Why I’m *not* recommending Janus as your default (even though it’s good)
Janus is legit, and its plugin model is handy. But:
- It’s **GPLv3**. citeturn0search1turn0search5  
In enterprise healthcare, that can become a legal procurement slog. Also you’ll still do a lot of product engineering around it.

(Yes, there are commercial licensing/support options for Janus. citeturn0search13 But you asked open-source and you sound like you want to ship, not negotiate licenses.)

---

## Why I’m not defaulting to Jitsi either
Jitsi is proven, but it’s an ecosystem (Prosody/Jicofo/JVB/Jigasi, etc.) and tends to pull you into “run the whole Jitsi world” mode. citeturn0search2turn0search6  
That can be totally fine. It’s just more moving parts than LiveKit for what you described.

---

## The actual short answer
- **Default SFU recommendation:** **LiveKit** citeturn0search0turn0search4  
- **If you need ultimate customization:** **mediasoup** citeturn0search7turn0search3  
- **If legal allows GPLv3 and you like gateway/plugin patterns:** Janus (not my first pick here) citeturn0search1turn0search5  

That’s the stack choice. The rest is just implementing a clean “one SIP bridge per PSTN conference” pattern so your DC doesn’t drown in SIP messages every time a browser user sneezes.

### User (2026-01-09 10:53:10)

Is there a free open-source option?

### Assistant (2026-01-09 10:53:11)

search("Is there a free open-source option?")

### Assistant (2026-01-09 10:53:11)

Free and open-source **yes**, but with important legal/ops tradeoffs you absolutely need to understand before you pick one.

Here’s the realistic list of *free, open-source SFU or WebRTC-centric stacks* that let you *avoid SIP for WebRTC signaling*:

## 🧠 Truly open-source, no cost

### **1) Jitsi (JVB + ecosystem)**
- **Free, open-source (Apache 2.0).**
- **SFU:** Jitsi Videobridge (JVB).
- **Signaling:** XMPP (Prosody) + Jicofo — *not SIP* for browser clients.
- **PSTN/SIP gateway:** Jigasi (joins SIP participants into rooms).
- **Recording/BRB:** Jibri.
- **Pros:** Mature conferencing stack, lots of community tooling.
- **Cons:** More moving parts (Prosody + Jicofo + JVB + Jigasi + Jibri) to operate and harden for compliance.

Good if you want *a full conferencing ecosystem you can deploy yourself*.

### **2) Janus Gateway**
- **Free, open-source** (GPLv3).
- **SFU-ish:** VideoRoom plugin gives you SFU behavior.
- **Signaling:** JSON over WebSockets/HTTP (not SIP).
- **SIP plugin:** optional (but GPLv3 contagion if you link/use it).
- **Pros:** Very flexible gateway model; can bridge to SIP only where needed.
- **Cons:** GPLv3 license often triggers legal review in enterprise/healthcare environments.

Janus is a great tech stack if your lawyers are okay with GPLv3.

### **3) mediasoup**
- **Free, open-source (MIT).**
- **SFU core:** yes, but **no built-in signaling**.
- **You build the signaling/control plane** (WebSockets/REST, etc.).
- **Pros:** Very flexible if you want a *custom room control policy*.
- **Cons:** You must build the signaling + compliance control plane yourself.

Mediasoup is more like the *media engine* than a “turnkey conferencing product.”

### **4) LiveKit (open-source edition)**
- **Free, open-source (Apache 2.0).**
- **SFU + signaling:** yes.
- **License:** Apache 2.0 (friendly).
- **Pros:** Modern API, built with scaling in mind, native WebRTC clients talk via WebSockets (not SIP).
- **Cons:** Some features (HIPAA readiness docs, region pinning, support) are primarily marketed via LiveKit Cloud — but the server code itself is open-source.

LiveKit’s OSS server is perfectly usable without paying, you just operate it yourself.

---

## ⚠️ Licensing realities that actually matter

| Stack | License | Notes |
|------|---------|-------|
| **Jitsi** | Apache 2.0 | Very permissive, easiest for enterprise/legal. |
| **Janus** | GPLv3 | *Copyleft* — many orgs reject this for production infra. |
| **mediasoup** | MIT | Permissive — but you build more yourself. |
| **LiveKit OSS** | Apache 2.0 | Permissive and modern; production-ready. |

If compliance/legal is part of your requirements (and it is), **Apache 2.0 / MIT stacks are vastly easier to adopt** than GPLv3 stacks in enterprise environments.

---

## 🧩 How each fits your “no SIP signaling for video” requirement

- **Jitsi:** Browser + mobile clients use XMPP/WebSocket signaling, not SIP. SIP only exists on the edge via Jigasi.
- **Janus:** WebRTC peers talk JSON-over-WS; SIP plugin is purely optional.
- **Mediasoup:** You choose the signaling protocol — not SIP unless you wire it in.
- **LiveKit:** Native WebSocket signaling, no SIP for browser signaling.

All four let you keep SIP signaling out of the video client flow.

---

## 🏁 So the “free open-source best bet” hierarchy

### 🥇 **LiveKit (OSS)**
**Best combination of open-source, permissive license, built-in signaling, SFU scalability, and clean integration with your platform’s control plane.**  
You’ll run it yourself, connect your app’s auth/rooms, and bridge to SIP/PSTN only where necessary.

### 🥈 **Jitsi**
Good if you want a more *complete application platform* (rooms, moderators, recording bots) out-of-the-box.  
Just be prepared to operate a few services together and harden for compliance.

### 🥉 **Mediasoup**
Choose this if you want *maximum control* over every signaling and policy detail and are fine building that layer.

### 🟥 **Janus**
Technically strong, but often legally disqualified for enterprise healthcare because of GPLv3.

---

## 📌 Practical deployment pattern

Given your constraints:

1. **WebRTC clients talk natively** to your SFU (LiveKit or Jitsi), not SIP.
2. **Bridge a single SIP leg** from FreeSWITCH into the SFU when a PSTN caller needs to mix into the same conference.
3. **Control roles/rooms/permission** via your platform (auth server). SFU is just the media plane.
4. **Recording + observer roles** live at the SFU layer or via dedicated recorder/observer endpoints you spin up.

This keeps SIP confined to the PSTN bridge and removes it from the video signaling path entirely.

---

## 📍 Final concise recommendation (no BS)

- **First choice:** **LiveKit OSS** — permissive, modern, built-in signaling, designed for SFU use.
- **Second choice:** **Jitsi Videobridge + ecosystem** — mature conferencing stack with SIP gateway classes.
- **Custom build:** **mediasoup** if you love building control planes.
- **Avoid unless approved:** **Janus** (GPLv3 concerns).

---

No SIP signaling overhead on video except where you *explicitly* bridge to PSTN via FreeSWITCH. Open-source estate is perfectly capable of this model — you just choose the stack that matches your compliance and ops tolerance.

### User (2026-01-09 10:55:54)

Ok so give me details on how hard it is to roll me own LiveKit OSS for a platform that handles around 2500 concurrent video calls and 500 pstn calls mixed in as some conferences, some standalone?

### Assistant (2026-01-09 10:55:55)

{"search_query":[{"q":"LiveKit server capacity concurrent participants sizing guide CPU bandwidth","recency":30},{"q":"LiveKit deployment guide Kubernetes ingress TURN coturn recommended","recency":365},{"q":"LiveKit SIP server documentation capacity scaling","recency":365},{"q":"LiveKit egress recording scaling documentation","recency":365}],"response_length":"short"}

### Assistant (2026-01-09 10:55:59)

Rolling your own **LiveKit OSS** for **~2500 concurrent video calls** plus **~500 PSTN calls (some mixed into conferences)** is *very* doable… if you treat it like a real-time media system and not like “another stateless web service with autoscaling dreams.”

Below is what’s actually hard, what’s merely annoying, and what you’ll need to size.

---

## 0) One constraint that drives everything
**In LiveKit, a room must fit on a single media node.** LiveKit scales by distributing *rooms* across nodes, but a given room doesn’t span multiple nodes. citeturn0search0

So your capacity planning is really two problems:
1) **Total concurrent load** (all rooms combined)
2) **Largest room / busiest conference** (must fit on one node)

---

## 1) Components you will operate (minimum viable “production-ish”)
### Core
- **LiveKit Server (media + signaling)**  
- **Redis** (coordination for distributed setup)
- **TURN** (either LiveKit embedded TURN/TLS or standalone coturn; you still need certs and real UDP reachability)

### Edge
- **HTTPS/WSS endpoint + TLS termination** (LB / Ingress). LiveKit explicitly calls out you need a proper domain + trusted SSL cert for SDKs. citeturn0search1
- **Firewall / UDP ports**: if you use LiveKit SIP server, both SIP signaling and a media port range must be reachable. citeturn0search8  
  (Even without SIP server, your LiveKit media nodes still need UDP port ranges open for WebRTC RTP/RTCP.)

### Recording (if required)
- **LiveKit Egress** runs separately when self-hosting. citeturn0search3  
- Composite recording uses **headless Chromium** for “room composite,” which has… personality… under load. citeturn0search9

---

## 2) Your workload translated into sizing terms
You said: **2500 concurrent video calls**. That phrase is ambiguous in the only way that matters (concurrency math):

### Case A: “Call” = 1:1 session
- 2500 calls ⇒ **5000 participants**
- Each participant publishes 1 video + 1 audio track

### Case B: “Call” = multi-party room
- You must size for the **largest room**, because it must fit on one node. citeturn0search0

**PSTN: 500 calls**
- If those are mostly audio-only: easy-ish from a bandwidth standpoint.
- If they’re mixed into video rooms: you should bridge **one PSTN/SIP leg per conference** (or a small fixed number), not one per browser participant event. That keeps SIP overhead bounded.

---

## 3) Network bandwidth (the real bottleneck most teams pretend isn’t)
SFU CPU is rarely your first limiter. **Egress bandwidth is.**

### Back-of-napkin formula (SFU relay)
For a room:
- **Ingress** ≈ Σ(publish bitrates)
- **Egress** ≈ Σ(publish bitrate × number of subscribers per track)

For 1:1 calls:
- Each call: two publishers; each publishes to one subscriber
- If average video bitrate is **1.5 Mbps** (typical-ish for 720p with modern codecs, but your reality may differ):
  - Per call ≈ **3 Mbps ingress + 3 Mbps egress**
  - 2500 calls ⇒ **7.5 Gbps ingress + 7.5 Gbps egress**
  - Add overhead, audio, retransmits, TURN-relay penalty, etc. You’re quickly in **10–20 Gbps aggregate** territory.

This alone pushes you toward:
- **Multiple media nodes with 10/25Gb NICs**, or a lot of 10Gb nodes
- Careful placement (don’t hairpin traffic through LBs that can’t pass UDP cleanly)

If you expect **1080p** or high-motion ASL video where you refuse to let bitrate drop, multiply pain accordingly.

---

## 4) CPU expectations (less scary, still non-trivial)
LiveKit is not transcoding (unless you build pipelines that do). SFU work is:
- SRTP encryption/decryption
- packet forwarding, pacing, congestion control interactions
- simulcast layer selection
- NACK/PLI/RTX handling

What spikes CPU:
- Big rooms (many subscribers per publisher)
- Heavy packet loss (retransmits)
- TURN-relayed sessions (more overhead)
- Composite recording with Chromium (that’s a whole separate “why is my node on fire” domain) citeturn0search9

---

## 5) The “hard parts” list (ranked by how much they ruin your week)

### Hard (real engineering)
1) **NAT traversal reliability** (TURN, ICE failures, hospital networks that hate UDP)
2) **Egress/recording architecture**: scale and isolate Egress, storage pipelines, encryption-at-rest, retention hooks (Egress is separate in self-hosting) citeturn0search3
3) **Capacity management with the “room fits on one node” constraint** citeturn0search0
4) **Observability that actually helps** (RTP stats, jitter, loss, MOS-ish measures, per-hop debugging)

### Annoying (ops)
- TLS + domains + ingress rules (non-negotiable) citeturn0search1
- UDP firewall rules and port ranges
- Kubernetes networking choices (hostNetwork vs alternatives). Some ecosystems recommend hostNetwork for media servers and note the operational/security tradeoffs. citeturn0search4

### Easy (if your team is competent)
- Standing up LiveKit + Redis
- Token auth and room provisioning from your platform

---

## 6) How many nodes for your target? A realistic starting estimate
Because bitrate assumptions dominate, here’s a **reasonable starting planning band** for **Case A (2500 1:1 calls = 5000 participants)**:

- Assume **~1–2 Mbps average video publish bitrate** (ASL may skew higher if you protect quality).
- Expect aggregate relay bandwidth roughly **8–16 Gbps each direction**, plus overhead.
- With **10Gb NICs**, you’ll likely want something like:
  - **8–16 media nodes** to have headroom (and to avoid single-node saturation during uneven load)
- With **25Gb NICs**, you can reduce node count, but you still want redundancy and room-spread.

This is not a promise, it’s an engineering estimate. The correct move is to run LiveKit’s own benchmarking guidance and stress profiles matching your codec/bitrate/room shapes. citeturn0search0

---

## 7) PSTN mixing with FreeSWITCH without SIP-signaling your video
Your requirement is sane:
- **WebRTC clients**: connect to LiveKit using LiveKit signaling (WSS).
- **PSTN/SIP**: stays in OpenSIPS/FreeSWITCH land.
- **Bridge**: create **one (or a small fixed number) of SIP participants per room** to tie PSTN audio into the LiveKit room.

If you instead make every WebRTC participant a SIP dialog, you’re voluntarily choosing datacenter pain. You already know that.

---

## 8) A practical rollout plan (so you don’t “big bang” this)
1) **Decide your “call shapes”**: percent 1:1 vs multi-party, typical max room size, expected bitrate floors for ASL.
2) Build a **benchmark scenario** that matches reality (number of tracks, subscribers, bitrate targets). LiveKit explicitly frames benchmarks around tracks/subscribers/data per subscriber. citeturn0search0
3) Stand up:
   - LiveKit cluster + Redis
   - TURN
   - Egress (separately) citeturn0search3
4) Add PSTN bridge per room via FreeSWITCH (or LiveKit SIP server if you choose that path, but you don’t have to). citeturn0search8
5) Load test with:
   - typical ASL video bitrate targets
   - hospital-like network profiles (loss, jitter, UDP blocking)
   - observer joins and recording

---

## Bottom line
**Hardness level:** medium-high, but not exotic. The “gotchas” are **networking, recording scale, and the single-node-per-room constraint**. citeturn0search0turn0search3turn0search1

If your 2500 “calls” are mostly 1:1, LiveKit is an excellent fit. If you have large multi-party rooms, your biggest planning variable becomes “how big is the biggest room,” because that dictates your largest node class. citeturn0search0

### User (2026-01-09 11:00:32)

I don't understand what you mean by single node per room - are you saying I would need a LiveKitOss server for every single call?

Currently we have 10 of these media servers with our current product and 2 stun servers that handle the sip signaling for the video media servers and the pstn that flows through opensips and freeswitch

### Assistant (2026-01-09 11:00:33)

{"search_query":[{"q":"LiveKit room placed on a single node each room cannot span multiple nodes","recency":30},{"q":"LiveKit distributed rooms across nodes single node per room redis","recency":365},{"q":"LiveKit server architecture room assignment one node per room","recency":365}],"response_length":"short"}

### Assistant (2026-01-09 11:00:36)

No. I’m **not** saying “one LiveKit server per call.” I’m saying something more annoying and more useful:

## What “single node per room” actually means
In a **distributed LiveKit cluster**, you run **many identical LiveKit server instances (nodes)** behind a load balancer, coordinated via **Redis**. LiveKit can handle **many rooms across many nodes**, but **each individual room is hosted on exactly one LiveKit node**. citeturn0search20turn0search0

LiveKit’s own benchmarking docs say it plainly: it “can scale to many simultaneous rooms” across nodes, **however, each room must fit within a single node**. citeturn0search20

### Practical translation
- You **do not** need “a LiveKit node per call.”
- You need **enough nodes so that:**
  1) your *total* number of rooms fits across the fleet, and  
  2) your **largest room** (biggest conference) fits on one node.

Think of it like:  
- **Rooms are sharded across nodes**, not spread across nodes.

---

## How this compares to what you have now (10 media servers + 2 STUN)
Your current setup sounds like a classic “pool of media servers” model:
- Calls are distributed across the 10 media servers
- STUN helps traversal
- SIP signaling is handled by OpenSIPS/FreeSWITCH for PSTN and likely for your video product too

LiveKit’s equivalent would be:
- **N LiveKit media nodes** (your “10 media servers” analog)
- **Redis** for cluster coordination (this is the “distributed brain”)
- **TURN/STUN** for NAT traversal (you’ll still want this; STUN-only is not enough in healthcare networks)
- Your platform controls who joins what room, roles, recording, observers, etc.

And LiveKit explicitly supports this distributed mode: when Redis is configured, it switches to a distributed setup and nodes use Redis as shared data store/message bus; nodes report stats and make routing decisions. citeturn0search0turn0search14

Also, the **Room Service API is distributed**, meaning any node can serve management requests about any room/participant/track. citeturn0search6

---

## So do you need 2500 LiveKit servers for 2500 calls?
No. You need a **cluster** sized for your concurrency and room shapes.

### Example mental model
If your workload is mostly **1:1** calls, rooms are tiny:
- Each room: 2 participants
- A single node can host a lot of tiny rooms (until bandwidth is saturated)

If you have **big conferences**, the limiting factor becomes the biggest room:
- If you have a 20-person ASL conference with observers and recording, that room’s egress load can be heavy.
- That whole room must sit on **one** node. citeturn0search20

So your scaling problem is:
- “How many total rooms?” (fleet capacity)
- “How big is the biggest room?” (node class requirement)

---

## What you should do with your existing FreeSWITCH/OpenSIPS world
You keep it. You just stop using SIP as the signaling plane for video users.

The clean pattern is:
- WebRTC video users connect to LiveKit using LiveKit signaling (WSS)
- PSTN/SIP continues through OpenSIPS/FreeSWITCH
- For “mixed” conferences, you add **a bounded number of SIP/PSTN bridge participants** into the LiveKit room (or vice versa), rather than forcing every WebRTC participant to be a SIP dialog

That preserves your “don’t SIP-signal video” requirement.

---

## The only numbers that matter next (and you already have them somewhere)
To translate your current “10 media servers” into LiveKit nodes, you need:
- Typical video resolution/bitrate targets (ASL often needs stable quality)
- Percent **1:1 vs multi-party**
- **Largest room size** you expect (including observers)
- How often users are TURN-relayed (hospital networks are… charming)

Without those, any node count guess is fantasy cosplay.

But the conceptual answer is firm: **a LiveKit cluster looks like your current pool of media servers**, not one server per call, with the extra constraint that **a single conference room can’t be split across multiple nodes**. citeturn0search20turn0search0

### User (2026-01-09 12:19:58)

would this architecture be able to support a multicloud region/multidatacenter region active/active scenario?

### Assistant (2026-01-09 12:19:59)

{"search_query":[{"q":"LiveKit self-host multi-region active active redis federation support","recency":14},{"q":"LiveKit server multi region deployment Redis requirements","recency":365},{"q":"Jitsi Videobridge multi region active active architecture","recency":365},{"q":"mediasoup multi region SFU active active","recency":365}],"response_length":"short"}

### Assistant (2026-01-09 12:20:03)

Yes, **but “active/active multi-cloud, multi-region” is not a checkbox**. It’s an architecture choice with some sharp edges, especially for WebRTC where *latency and NAT reality* dictate what “active/active” can even mean.

## What “active/active” can mean for an SFU
There are basically **two sane models**:

### Model 1: **Active/active by region, rooms pinned to a region**
- You run a **separate LiveKit cluster per region** (each with its own Redis, TURN, media nodes).
- Users connect to the **nearest region**.
- A given room lives entirely in one region (best latency, simplest).
- If Region A dies, new sessions go to Region B; existing rooms die unless you built a “failover room” experience in the app.

This is the most realistic “active/active” for healthcare realtime video.

### Model 2: **Single logical cluster stretched across regions**
- LiveKit supports **distributed setups** using Redis as room data store + message bus. citeturn0search0turn0search7
- In theory, nodes in different regions can participate in one distributed cluster.
- In practice, the **cross-region latency to Redis and inter-node coordination can become a performance and reliability problem**. (Even LiveKit users report multi-region pain when Redis is far away, with ~140ms latencies mentioned in a real issue report). citeturn0search16

This model is possible, but it’s the one that produces “we built a distributed system and now we live in it.”

---

## LiveKit specifically: can it support geo-distributed setups?
LiveKit’s docs explicitly mention **distributed multi-region setups** for self-hosting. citeturn0search7turn0search0  
And their Kubernetes and deployment guides assume a multi-node cluster with Redis and a secure domain/SSL termination. citeturn0search3turn0search4

So **yes**, LiveKit can be deployed in a geo-distributed manner, but the best-practice interpretation for “active/active” is usually **Model 1** (regional clusters) unless you enjoy debugging cross-region Redis pub/sub behavior. citeturn0search16

---

## The hard constraint you can’t dodge
**A room’s media must be handled by a specific media node** (rooms are not “striped” across regions). That’s why region-pinning rooms is the sane plan. citeturn0search0

So “active/active” does **not** mean:
- one room seamlessly spanning multiple regions with participants attached to their nearest SFU without extra machinery.

It means:
- many rooms distributed across many nodes, and you decide *which region owns which room*.

---

## What you need to make Model 1 work (the sane one)
### Per region
- LiveKit server pool + Redis (regional)
- TURN (regional)
- Egress/recording services (regional, or centralized with careful bandwidth/security)
  - LiveKit Egress uses Redis queues to coordinate with LiveKit. citeturn0search8

### Global
- **Geo DNS / anycast / latency routing** to send clients to the nearest region
- Your **control plane** decides the room’s “home region” and issues tokens accordingly

### Failover behavior
- **Active/active** becomes: *both regions take traffic concurrently*, but **failover is at the session level**, not “transparent continuation of the same room.”
- You can design “failover rooms” or “rejoin in backup region” flows in the app. Most products do this.

---

## Multi-cloud nuance
Multi-cloud adds the extra fun of:
- different LB behaviors for UDP
- different NAT traversal success rates
- different egress billing and bandwidth caps
- different operational knobs for autoscaling and host networking

LiveKit’s Kubernetes guidance is generic across providers, but WebRTC media servers commonly benefit from host-level networking decisions (and the tradeoffs are real). citeturn0search3turn0search4

---

## If you *really* need multi-region same-room behavior
Then you’re talking about **bridge cascading / SFU federation** style architectures (e.g., Jitsi’s research and blog posts around geographically distributed conferencing and cascaded bridges). citeturn0search5turn0search20  
That’s a different tier of complexity and usually not worth it unless you have a global meeting product with lots of large rooms spread across continents.

If you want a component stack that leans into “multi-router interconnect,” mediasoup has mechanisms like router interconnects (PipeTransport is commonly referenced in community/academic discussions), but that’s a “build your own federation logic” path. citeturn0search6turn0search2

---

## Bottom line
- **Yes**: you can do **active/active multi-region** with LiveKit OSS.
- The sane, scalable way is **regional clusters + room region pinning** (Model 1). citeturn0search7turn0search0
- A single stretched “one cluster across regions” is possible but fragile because it leans hard on cross-region Redis coordination latency and reliability. citeturn0search16

If you want this to work in healthcare conditions (hospital networks, strict uptime expectations), you pick the boring path: **per-region clusters, global routing, and session-level failover semantics**.

### User (2026-01-09 12:25:41)

Let's just say for a second the system I am building needs to be able to do this:

"Then you’re talking about bridge cascading / SFU federation style architectures (e.g., Jitsi’s research and blog posts around geographically distributed conferencing and cascaded bridges).  ￼
That’s a different tier of complexity and usually not worth it unless you have a global meeting product with lots of large rooms spread across continents."

Budget is not a constraint in this case. I truly need a global, scalable, active/active solution

### Assistant (2026-01-09 12:25:42)

{"search_query":[{"q":"Jitsi Videobridge cascading bridges Octo architecture geo distributed conferences","recency":30},{"q":"LiveKit multi-region federation rooms across regions redis latency guidance","recency":365},{"q":"mediasoup PipeTransport multi-router federation geo distributed","recency":365},{"q":"WebRTC SFU cascading architecture multi region active active conference design","recency":365}],"response_length":"medium"}

### Assistant (2026-01-09 12:25:45)

Yes. You can build **true global active/active**. You just can’t build it by wishing really hard at a “single cluster stretched across regions” and hoping physics behaves. What you actually want is **federation (cascading SFUs)**: local SFUs per region, stitched together *per room* over bridge-to-bridge links.

That’s the architecture Jitsi has publicly described and implemented (bridge cascading / Octo and newer “relays”), and it’s the most battle-tested open model of what you’re asking for. citeturn0search0turn0search7turn0search31

Below is the practical blueprint, plus what it costs in complexity.

---

## The only global active/active model that stays sane

### Regional cells (active/active everywhere)
Run a full “cell” in each region / datacenter:

- **Edge**: WSS/HTTPS signaling + auth/token service (your control plane)
- **SFU pool**: media servers (JVBs, or your SFU of choice)
- **TURN**: local TURN clusters (you will need them in healthcare networks)
- **Recording/egress**: regional recording workers + storage pipeline
- **SIP/PSTN bridge**: local gateways into FreeSWITCH/OpenSIPS (or vice versa)

Users attach to the **nearest cell**. The room exists globally, but participants attach locally.

### Room-level federation (the key move)
For each room, you form **a federation graph**:
- Participants in Region A connect to SFU-A
- Participants in Region B connect to SFU-B
- SFU-A and SFU-B exchange media over **bridge-to-bridge relays**
- Each SFU forwards only what’s needed to its local participants (ideally)

This is “SFU cascading.” citeturn0search0turn0search7turn0search3

---

## Why this beats “one global cluster”
LiveKit’s distributed mode uses Redis for room data + message bus when running multi-node, and that’s great for redundancy and spreading rooms across nodes. citeturn0search1turn0search8  
But **stretching that across regions** drags you into cross-region coordination and pub/sub failure domains (and yes, people have had multi-region incidents tied to Redis/pubsub/RPC behavior). citeturn0search5

Federation avoids that by keeping **real-time room control and media local**, with **purpose-built SFU-to-SFU media links**.

---

## The most “ready” open-source path: Jitsi cascading (Octo/Relays)
Jitsi has described and shipped bridge cascading explicitly:
- “Bridge cascading is back” (geo-location, cascaded bridges, architecture + experiments). citeturn0search0  
- Older but still relevant explainer material on geographical cascading. citeturn0search4turn0search27  
- They also discuss how cascading works at the bridge-to-bridge level and how simulcast streams get forwarded across bridges in their architecture discussions. citeturn0search7  
- JVB docs include “relays” (cascaded bridges) using ICE + DTLS-SRTP between bridges, so it can run over untrusted networks. citeturn0search31  

### What you get with this approach
- **True global active/active**: every region serves local users.
- **Large rooms**: can be distributed across bridges (you’re no longer constrained to “one room fits on one node”).
- **Latency improvement**: local last-mile to nearest SFU.

### What it costs you
- **Inter-region bandwidth** becomes a first-class expense and design input (no free lunch).
- **Relay selection logic** matters (which bridges are in the conference, and who connects where).
- **Operational complexity** goes up: you now operate a federation layer.

Also worth knowing: cascading has been disabled/changed in the past in some contexts because WAN bandwidth costs can explode if you forward too much between bridges (research literature calls this out when discussing cascaded meshes). citeturn0search23

---

## Alternative “buildable” path: mediasoup federation
If you want to implement your own federation logic, mediasoup gives you primitives:
- `router.pipeToRouter()` / **PipeTransport** interconnects routers and is explicitly intended for router-to-router media piping (even across hosts). citeturn0search2turn0search9turn0search13  

This is powerful, but it’s not a turnkey global conferencing product. You build:
- signaling + room state
- router graph management
- admission control
- routing policy (who gets which streams across regions)

If you said “budget is unlimited,” the hidden constraint is still “engineering time and risk,” not just money.

---

## How you integrate PSTN/SIP cleanly (with federation)
You already have OpenSIPS + FreeSWITCH. Keep it.

Per room, you do:
- **One PSTN gateway leg per conference** into *one* SFU cell (ideally the cell closest to the PSTN ingress or to the “conference owner” region).
- That audio is then federated to other regions as needed.

Do **not** do “PSTN leg per region” unless you like echo paths, drift, and paying twice.

---

## Recording and “observer” in a federated world
This is where healthcare requirements bite, but it’s solvable.

### Recording
Two sane patterns:
1) **Single authoritative recorder per room** (best for chain-of-custody)
   - Choose a “home region” for recording
   - Recorder subscribes to the federated mix/streams there
2) **Regional recording** (only if required for data residency)
   - More storage and reconciliation complexity
   - More policy surface area

### Observer
Observer is just a constrained participant:
- Connect observer to nearest SFU
- Apply policy: subscribe-only, restricted tracks, hard-mute publishing
- Audit everything (your platform owns this)

Federation doesn’t break observer. It just means the observer may subscribe to streams that originate in another region, which is exactly what the SFU-to-SFU links are for.

---

## Straight recommendation given “must be global active/active”
If you truly need **global, active/active, room-spanning** conferencing:

### Most proven open-source direction
**Jitsi Videobridge with cascading/relays (Octo-style federation)**. citeturn0search0turn0search31turn0search7  

### Most flexible “build your own Zoom backbone” direction
**mediasoup + PipeTransport federation**, but expect substantial custom engineering. citeturn0search2turn0search13turn0search9  

### LiveKit note (important)
LiveKit **Cloud** claims “global mesh architecture” and “unlimited participants” in its managed offering marketing. citeturn0search20  
But if you’re asking about **LiveKit OSS** doing true cross-region room federation, you should treat that as *not the default OSS path* and not assume it’s equivalent to Jitsi-style cascading without verifying the exact OSS capabilities you’d be deploying.

---

## The unavoidable tradeoff you’re signing up for
Global active/active federation buys you latency and scale, and then sends you an invoice for:
- **WAN bandwidth** (bridge-to-bridge traffic)
- **Complexity** (routing policies, failure modes, observability)
- **Consistency** (who is “the room authority,” where recordings live, how moderation propagates)

Budget not being a constraint helps, but it doesn’t repeal math or networking.

If you build this, the design target is “regional cells + per-room bridge federation,” not “one giant multi-cloud cluster.” That’s the difference between a platform and a distributed outage generator.

