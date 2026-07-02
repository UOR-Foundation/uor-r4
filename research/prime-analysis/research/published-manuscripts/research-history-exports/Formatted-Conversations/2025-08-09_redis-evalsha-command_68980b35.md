# Redis EVALSHA command

- **Conversation ID:** `68980b35-76f0-8331-93cb-9c39dbcc12fb`
- **Date:** 2025-08-09

---

### User (2025-08-09 23:00:11)

what is redis evalsha?

### Assistant (2025-08-09 23:00:12)

`EVALSHA` is a Redis command used to execute a **Lua script** that has already been loaded into Redis by its SHA1 hash.  

Here’s the breakdown:  

---

### **1. The basics**
- Redis supports running Lua scripts through the `EVAL` command.
- When you first use `EVAL`, you send the **entire script text** to Redis, and Redis:
  1. Stores the script internally.
  2. Computes a **SHA1 hash** of the script text.
  3. Executes it.
- Once loaded, you don’t need to send the whole script again — you can just reference it by its SHA1 hash using `EVALSHA`.

---

### **2. Why use `EVALSHA` instead of `EVAL`?**
- **Performance:** Avoids resending the full script over the network every time.
- **Efficiency:** Saves parsing time — Redis already has the script stored.
- **Bandwidth:** Useful if the script is large and executed many times.

---

### **3. Command format**
```redis
EVALSHA sha1 numkeys key1 key2 ... arg1 arg2 ...
```
Where:
- `sha1` = the SHA1 checksum of the script.
- `numkeys` = number of keys the script will access.
- `key1 key2 ...` = keys passed to the script.
- `arg1 arg2 ...` = additional arguments passed to the script.

---

### **4. Example**
```redis
127.0.0.1:6379> EVAL "return redis.call('set', KEYS[1], ARGV[1])" 1 mykey "hello"
OK

# Get SHA1 hash of the script (done automatically by Redis when you use EVAL)
# Let's say it's: f4d3a59df1234d93d02e74b2c3f87b7b3cb9d2e7

127.0.0.1:6379> EVALSHA f4d3a59df1234d93d02e74b2c3f87b7b3cb9d2e7 1 mykey "world"
OK
```
Here, the second call skips sending the script text — it just references the stored script by hash.

---

### **5. Common gotchas**
- If the script was never loaded with `EVAL` (or `SCRIPT LOAD`) in the current Redis instance, `EVALSHA` will fail with:
  ```
  (error) NOSCRIPT No matching script. Please use EVAL.
  ```
- Scripts are stored **in memory** — if Redis restarts or scripts are flushed with `SCRIPT FLUSH`, you must reload them.

---

### **6. Related commands**
- `SCRIPT LOAD "script"` → Loads the script into Redis and returns its SHA1 without executing it.
- `SCRIPT EXISTS sha1 [sha1 ...]` → Checks if a script exists in cache.
- `SCRIPT FLUSH` → Clears all cached scripts.

---

If you want, I can also explain **how Django or Celery might be using `EVALSHA` under the hood with Redis**, since it’s often used for rate limiting, locks, or atomic operations without transactions. That might tie directly to the “taking forever” issue you mentioned earlier.

### User (2025-08-09 23:02:58)

yes, mget was taking forever, and it seems that evalsha runs right before long "uninstrumented time periods" in our observability platform, that I've been told are due to long running transactions sometimes... so I'm wondering if mget and evalsha are somehow conflicting with and/or triggering eachother and causing or amplifying our application wait times

### Assistant (2025-08-09 23:02:59)

Short answer: yes—`EVALSHA` can absolutely amplify (or cause) long waits that make `MGET` “take forever,” but it’s indirect. A Lua script runs **atomically** and **blocks the Redis event loop** until it finishes. While it’s running, **no other command** (including `MGET`) is served. If the script is slow (touches many keys, scans, manipulates big values, or hits Redis’s Lua time limit path), everything else queues behind it. From your app’s perspective this looks like a long “uninstrumented” or “transaction” stall.

Here’s how to reason about it and pin it down fast.

# What typically goes wrong

1) Blocking from Lua  
- A single slow `EVALSHA` blocks the server thread. Any concurrent `MGET` waits in the socket backlog.  
- Triggers: scripts that iterate over many keys/large values, `SCAN`-like loops, big JSON blobs, long string concat, per-key `GET`/`SET` in loops, or misusing `KEYS` (worst case).  
- If a script exceeds `lua-time-limit` (default 5s), Redis enters the **“busy script”** state: it keeps blocking until the script returns or you `SCRIPT KILL`. During this, all clients stall.

2) Payload and memory pressure  
- A large `MGET` returning megabytes can spike network and allocator time; right after that, a script runs and blocks, compounding tail latency.  
- If you’re near `maxmemory`, evictions or COW (fork for RDB/AOF) can add latency bursts that make any command appear slow.

3) Cluster routing edge cases  
- In Redis Cluster, scripts must operate on keys in a **single hash slot**. If your client/library tries cross-slot patterns, you’ll see errors or retries; some frameworks fall back to non-script paths inconsistently, producing odd latency patterns.

4) App-level interaction  
- Many Django/Celery/ratelimit/lock libs use Lua (via `EVALSHA`) for atomic counters, sliding windows, or locks. If that script becomes hot and slow, the **entire cache plane** suffers—your innocent `MGET`s back up behind it, and your APM shows “uninstrumented” time because the thread is just blocking on I/O.

# What to check (low-risk, high-signal)

Run these against the affected Redis node(s):

**1) Slowlog for smoking guns**
```bash
SLOWLOG GET 128
```
Look for `EVAL`/`EVALSHA` entries with big µs values and large `argc` or obvious loops. If you see sustained entries >5–10 ms (let alone ms→sec), that’s your culprit.

**2) Command mix & latency**
```bash
INFO commandstats
```
Compare `cmdstat_evalsha` vs `cmdstat_mget` (calls, usec_per_call). High `usec_per_call` on `evalsha` explains the backlog.

**3) Latency doctor (server-side stalls)**
```bash
LATENCY DOCTOR
LATENCY HISTOGRAM
```
Look for events labeled `command`, `fork`, `aof-write`, `rdb-unlink-temp-file`, `active-defrag-cycle`. If `command` spikes align with your outages, it’s likely a blocking script or jumbo `MGET`.

**4) Busy scripts / time limit**
```bash
CONFIG GET lua-time-limit
```
If you ever saw `(BUSY) Lua script is running` errors in clients, you’re hitting this. Also check:
```bash
INFO stats
# look at "instantaneous_ops_per_sec", "total_error_replies"
```

**5) Script inventory**
```bash
SCRIPT EXIST <sha1...>
SCRIPT LIST   # (Redis 7+)
```
Identify which scripts are present; match SHA1s to your frameworks (django-redis, celery, rate-limiter, redlock, etc.).

**6) Memory and fragmentation**
```bash
INFO memory
```
High fragmentation or near `maxmemory` increases allocator pauses.

# App/library patterns that commonly cause this

- **Rate limiting / sliding window**: Lua loop over many timestamps/keys per request.  
- **Distributed locks (redlock-style)**: frequent `SET NX` + `PEXPIRE` plus Lua cleanup; fine when tiny, but can become hot under load.  
- **Cache stampede protection**: Lua to “single-flight” rebuilds; under thundering herd, scripts become long-running.  
- **Improper use of `KEYS`** inside Lua (never do this in production).  
- **Huge `MGET`**: fetching thousands of keys or multi-MB values per call; even if fast individually, any concurrent slow script turns the queue into molasses.

# Minimal, decisive experiments

1) **Measure per-command latency now**  
Enable slowlog if it isn’t:
```bash
CONFIG SET slowlog-log-slower-than 1000    # 1 ms
CONFIG SET slowlog-max-len 2048
```
Run your traffic for a few minutes; re-check `SLOWLOG GET`.

2) **Isolate the script**  
Temporarily disable the suspected middleware (rate limiter/lock) on a canary instance or route its traffic to a separate Redis. If `MGET` latency normalizes, the interaction is confirmed.

3) **Cap MGET size & value size**  
Batch `MGET` to ≤512 keys (often far less) and ensure values are <64–256 KB. Watch p95/p99.

4) **Client-side timeout & backpressure**  
Set strict Redis client timeouts and connection pool limits so blocked calls don’t cascade into “uninstrumented time.”

# Fixes (ordered by impact)

- **Make scripts O(1) in key count.** Avoid loops over arbitrary sets. Use **single-key data structures** (hashes/zsets) where one op updates state.  
- **Move heavy logic out of Lua.** Use a short script only for the atomic “check-then-set” boundary; do the rest in app code.  
- **Shard hot script keys** by hash tag to multiple slots/nodes to reduce single-thread contention.  
- **Eliminate `KEYS` and unbounded scans** inside scripts.  
- **Tune `lua-time-limit`** only after you’ve reduced complexity; increasing it just prolongs blocking.  
- **Right-size `MGET`.** Prefer pipelines of `GET`s or fetch smaller aggregates (e.g., a single `HGETALL` of a compact hash) instead of thousands of discrete big values.  
- **Separate planes.** Put lock/rate-limit scripts on Redis-A; cache reads/writes on Redis-B. Different nodes = no queue coupling.  
- **Watch RDB/AOF settings.** If you see `fork`/`aof-write` latency in `LATENCY DOCTOR`, adjust `save` cadence, consider `appendfsync everysec`, or offload AOF/RDB to replicas.

# A concrete “Django stack” suspicion list

- **django-redis**: uses Lua for “atomic get-or-set” and TTL management. Under stampede, those scripts get hot.  
- **Celery / beat**: distributed lock scripts for schedules.  
- **Rate limiters** (django-ratelimit, DRF throttling helpers): sliding window scripts that touch many keys.  
Any one of these, when slow, will block your cache `MGET`s.

---

If you want, give me:  
- a sample `SLOWLOG GET 32` snippet,  
- `INFO commandstats` (just the `cmdstat_evalsha` and `cmdstat_mget` lines), and  
- whether you’re on single-node Redis or Cluster.  

I’ll pinpoint the offending script form (counter window, lock cleanup, or stampede control) and sketch an O(1) rewrite that won’t block your `MGET`s.

### User (2025-08-09 23:05:45)

we use azure cache for redis - according to our developers, it is only used in a pub/sub format, although I'm not sure how true that in anymore seeing as how a lot of our applications are currently using celery and the django 3 framework (including ws4redis) that even ties some on premise datacenters to cloud redis via expressroutes (that are not highly available) so I believe there is probably something going on in this stack

### Assistant (2025-08-09 23:05:46)

Yeah—your stack can absolutely create the symptoms you’re seeing, even if “we only use pub/sub” was once true. In practice:

- **Pure pub/sub doesn’t use `EVALSHA`.**  
  If you’re seeing `EVALSHA` in commandstats/slowlog, some library *is* loading/running Lua (likely Django cache, throttling, locks, or cache-stampede control), meaning your Redis isn’t “pub/sub-only” anymore.

- **A single slow Lua script blocks the server thread.**  
  While a script runs, **no pub/sub delivers, no `MGET` returns, no Celery broker ops progress**—everything queues. From the app/APM, this shows up as long “uninstrumented”/I/O wait.

- **ExpressRoute flaps amplify it.**  
  If the on-prem link drops, you get reconnect storms: thousands of resubscribes, Celery workers rebinding, Django clients re-`AUTH`/`HELLO`. If that surge coincides with a hot script, tail latency explodes.

Below is a concise triage plan tailored to Azure Cache for Redis + Django/Celery/ws4redis, then the likely culprits and fixes.

# Triage: confirm it in 15–30 minutes

1) **Prove that Lua is involved (or not).**  
   - `INFO commandstats` → check `cmdstat_evalsha` counts and `usec_per_call`.  
   - `SLOWLOG GET 128` → any `EVAL`/`EVALSHA` > 2–5ms is a red flag.  
   - If `EVALSHA` doesn’t appear at all, your stalls are probably network/connection storms (see ExpressRoute notes below).

2) **Correlate Redis-side stalls.**  
   - `LATENCY DOCTOR` / `LATENCY HISTOGRAM` → look for `command` spikes (busy scripts) or `fork` / `aof-write` / defrag events.  
   - `CONFIG GET lua-time-limit` → if you ever see BUSY warnings in clients, you’re tripping it.

3) **Check connection pressure & broker behavior.**  
   - `INFO clients` → very high client count or frequent `blocked_clients` during incidents = BRPOP consumers (Celery) + reconnections.  
   - Azure portal metrics: spikes in **Connected Clients**, **Server Load**, **Operations/sec**, and **Network In/Out** aligning with your stall windows strengthen the “reconnect storm + blocking script” story.

4) **Inventory who’s using scripts.**  
   - `SCRIPT LIST` (Redis 7) / `SCRIPT EXISTS <sha…>` → get the SHA1s.  
   - Cross-check your stack: `django-redis`, DRF throttling, rate limiters, “single-flight” cache stampede protection, distributed locks (e.g., redlock or custom) are typical sources.

# Likely culprits in your stack

- **Django cache (django-redis)**  
  Uses Lua for atomic “get-or-populate”/TTL adjustments. Under stampede, that script becomes hot.  
- **DRF throttling / rate-limiting middlewares**  
  Sliding-window counters implemented in Lua that iterate over multiple keys.  
- **Distributed locks** (task de-dup, leader election, cron guards)  
  Small scripts, but catastrophic if they loop or touch many keys.  
- **Celery**  
  As broker: primarily `BRPOP`/`XREAD` style ops (no Lua by default). But if you added lock/rate-limit around tasks, Lua enters.  
- **ws4redis / pubsub**  
  Pub/sub itself doesn’t use Lua, but it *suffers* when Lua blocks the server thread. Disconnects/resubscribes during ExpressRoute blips magnify load.

# Why “MGET takes forever”
Because **everything** waits behind the script. Even a modest 50–200 ms script at high QPS can push p95/p99 into seconds once queues form. If the script crosses `lua-time-limit`, it becomes a BUSY script, and Redis won’t process anything else until it returns (or you `SCRIPT KILL` if allowed).

# Architecture fixes (order by impact)

1) **Separate planes (do this even if you fix the script).**  
   - **Cache plane:** Django caching, any Lua-based atomics.  
   - **Broker plane:** Celery queues/streams.  
   - **Pub/Sub plane:** ws4redis/websocket notifications.  
   Use **separate Azure Cache for Redis instances** (or separate shards/DBs if you must), so a blocking script can’t freeze pub/sub and the broker.

2) **Eliminate unbounded Lua.**  
   - No loops over dynamic/large key sets; no `KEYS`; no scan-while-you-hold-the-event-loop.  
   - Rework to **O(1) writes** using a single hash/zset per logical entity.  
   - Keep Lua as a tiny atomic boundary; push heavy work to app code.

3) **Tame stampedes and rate limits without server blocking.**  
   - Prefer **SET NX PX** with short TTL for single-flight, and rebuild in app; or use **client-side** single-flight (e.g., per-process mutex) to reduce script pressure.  
   - For rate limiting, switch to **fixed-window counters** or ZSETs with bounded cardinality; avoid sliding-window loops in Lua.

4) **Provision for reconnection storms.**  
   - On Azure, size instances for **connection count** and **network throughput**, not just memory.  
   - Set client pool limits, jittered reconnect backoff, and reasonable timeouts so flaps don’t nuke the server.

5) **Network hygiene for ExpressRoute.**  
   - If that ER circuit isn’t HA, accept that it *will* create bursts. Either add HA or ensure the **pub/sub plane is entirely cloud-local** so websocket fan-out isn’t cross-ER.  
   - Keep on-prem writers/readers off the cache plane that runs Lua.

6) **Observability hardening.**  
   - Turn up `slowlog` length (e.g., 2048) and set 1–2 ms threshold during an incident window to capture the offender.  
   - Export `INFO commandstats` and `LATENCY DOCTOR` snapshots to your APM on a short cadence; correlate with app p95/p99.

# Quick decision tree

- **See `EVALSHA` in `commandstats` or `SLOWLOG`?**  
  → Fix scripts + separate planes (cache/broker/pubsub).  
- **No `EVALSHA`, but huge connection spikes & timeouts during incidents?**  
  → ExpressRoute reconnection storm. Add HA or move pub/sub local; tune client backoff and pool caps.  
- **`MGET` itself huge (thousands of keys / multi-MB values)?**  
  → Batch smaller; consolidate to hashes; reduce value size. Large replies + any blocking event = long waits.

# What I need to pinpoint it precisely
If you can paste (sanitized is fine):
- `INFO commandstats` (just the `cmdstat_evalsha`, `cmdstat_eval`, `cmdstat_mget`, and total ops lines),
- `SLOWLOG GET 64` around an incident,
- Azure metrics screenshot or numbers (Connected Clients, Server Load, Ops/sec, Network In/Out) for the same window,
- Whether your Azure tier is Standard/Premium/Enterprise and if clustering is enabled,

…I’ll tell you exactly which pattern you’re hitting (busy script vs. reconnect storm vs. jumbo `MGET`) and outline the minimal code/config changes to stop it without a risky refactor.

### User (2025-08-09 23:14:26)

yes we have evalsha's taking 5+ seconds with minutes of "uninstrumented time" following them. The mgets seemed to be initiated by eventhub consumers that were somehow having trouble connecting (or possibly were not disconnecting properly) from redis?

either way, yes redis has been split into: 1 cache for client websockets... including healthchecks i believe, and a second cache for celery worker jobs... unfortunately it seems that our celery cache is running into all of the problems now, mostly because (I think) the devs are running code that queries django admin directly to build reports (because they think it bypasses the database... but it obviously doesn't)

### Assistant (2025-08-09 23:14:27)

Got it—this lines up cleanly:

- A few `EVALSHA` calls run 5+ seconds → Redis’s single thread is blocked.
- While blocked, **all** Celery broker ops and your `MGET`s queue.
- When the script finally returns, Redis has a huge backlog; draining that queue + client timeouts/retries = the “minutes of uninstrumented time.”
- EventHub consumers (leaky connects or reconnect storms) plus Django-admin-driven report jobs make the surge worse.

Here’s a tight, surgical plan—first to stop the bleeding, then to fix root causes.

# Stop-the-bleed (today)

1) **Quarantine the offender.**
   - Stand up a **third** Azure Cache for Redis (temporary is fine).
   - Route the reporting jobs (or the component that calls that Lua) to this new instance.  
   Outcome: the Celery broker won’t freeze when that script blocks.

2) **Throttle Celery fan-out to avoid queue avalanches.**
   In Celery settings:
   ```python
   worker_prefetch_multiplier = 1
   broker_pool_limit = 50
   broker_heartbeat = 20
   broker_transport_options = {
       "visibility_timeout": 3600,
       "socket_timeout": 5,
       "socket_connect_timeout": 5,
       "max_connections": 1000,
   }
   ```
   If you use redis-py 4.x+, set client pool `max_connections` and `health_check_interval=30` so dead TCPs don’t pile up.

3) **Put hard guards around `MGET`.**
   - Cap list size: never fetch > 256–512 keys per call.
   - If values are big, batch in smaller pipelines or move to **hashes** so you can `HGETALL`/`HMGET` per object.

4) **Have the kill-switch ready.**
   - If you hit a “BUSY script” again and must recover the broker: `SCRIPT KILL` (works only if the script didn’t write); otherwise you’ll have to let it finish. Lowering `lua-time-limit` won’t fix root cause, but setting it to ~3000 ms can keep the backlog from going catastrophic while you triage.

# Pinpoint the exact script (1–2 hours)

Run on the Celery Redis during/after an incident:

```redis
SLOWLOG GET 256
INFO commandstats
LATENCY DOCTOR
```

On Redis 7:
```redis
SCRIPT LIST
SCRIPT GET <sha1>
```
Match the SHA1s from slowlog to source via `SCRIPT GET`. You’ll likely find one of:
- a rate-limit/sliding window loop,
- a cache single-flight script under stampede,
- a distributed lock cleanup loop,
- or a custom report helper touching many keys.

If not on Redis 7, log at the app layer: wrap the Redis client to log the call stack whenever `EVALSHA` is invoked so you can map SHA→code path.

# Root fixes (this week)

**A) Move reporting off the broker plane (best).**  
Those Django-admin report builders should **never** hit the Celery broker Redis. Options:
- Point reporting to a **read replica** of Postgres (or a dedicated reporting DB).
- If they need cache, give them their **own** Redis.
- If they need a queue, use **RabbitMQ** or **Azure Service Bus** (Celery supports both). Redis is a fine cache; it’s a fragile broker under script load.

**B) Remove unbounded Lua.**  
Rewrite the offending script to **O(1) keys**:
- No `KEYS`, no SCAN-loops.
- Represent state in a single **hash** or **zset** per logical entity, so one op updates it.
- Use Lua only for the atomic boundary; push iteration/aggregation to app code (or the DB).

**C) De-stampede the cache without Lua locks.**  
Prefer:
```redis
SET key value NX PX <ttl>
```
for single-flight, and rebuild outside Redis; or use per-process mutex. Lua single-flight that loops over dependents is the usual time bomb.

**D) Tame EventHub consumers.**
- Ensure they **close** Redis connections (pool return) and have jittered reconnect backoff.
- If they checkpoint offsets, don’t store them via heavy Lua in Redis; use the Event Hubs checkpoint store (Blob/ADLS) designed for it.

**E) Separate planes permanently.**  
You already split websocket vs Celery. Keep going:
- **Plane A (cache)**: Django cache, any Lua atomics.
- **Plane B (broker)**: Celery queues/streams (no Lua).
- **Plane C (pub/sub)**: ws4redis.  
Different Azure Cache instances (or at least different shards). This prevents cross-plane freeze-ups.

# Quick health checks you can automate

- Alert if `cmdstat_evalsha.usec_per_call > 2000` **or** `SLOWLOG` has `EVAL/EVALSHA > 100ms` 10+ times in 5 minutes.
- Alert on `Connected Clients` spike + `Ops/sec` drop (classic busy-script signature).
- Export `LATENCY LATEST` every minute; page on `command` spikes.

---

If you paste one `SLOWLOG` entry with the SHA and arg count, I’ll tell you exactly what pattern it is and give you a drop-in O(1) replacement sketch (hash/zset schema + 10–20-line Lua or pure-Redis ops) so you can rip out the blocking bit without a big refactor.

