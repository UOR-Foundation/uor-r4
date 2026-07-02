# Nginx Call Blocking Issue

- **Conversation ID:** `681e24fe-1424-8001-a18a-e9aa8a8e496a`
- **Date:** 2025-05-09

---

### User (2025-05-09 11:53:33)

Please evaluate the following nginx logs from our nginx docker container and tell me whe we cannot make phone calls from our system because it is being blocked and please tell me how to fix this in the config:

2025-05-09T15:43:08.414079090Z 172.16.102.139 - - [09/May/2025:15:43:08 +0000] "POST /freeswitch_api/fs/in/pstn/park HTTP/1.1" 404 0 "-" "python-requests/2.32.3" "-" 
2025-05-09T15:43:19.365711045Z 172.16.102.139 - - [09/May/2025:15:43:19 +0000] "PUT /freeswitch_api/channel/hangup HTTP/1.1" 404 74 "-" "python-requests/2.32.3" "-" 
2025-05-09T15:43:20.434088931Z 2025/05/09 15:43:20 [info] 576#576: *7 ModSecurity: Warning. Matched "Operator `PmFromFile' with parameter `scanners-user-agents.data' against variable `REQUEST_HEADERS:User-Agent' (Value: `Mozilla/5.0 zgrab/0.x' ) [file "/usr/local/owasp-modsecurity-crs/rules/REQUEST-913-SCANNER-DETECTION.conf"] [line "33"] [id "913100"] [rev ""] [msg "Found User-Agent associated with security scanner"] [data "Matched Data: zgrab found within REQUEST_HEADERS:User-Agent: mozilla/5.0 zgrab/0.x"] [severity "2"] [ver "OWASP_CRS/3.2.0"] [maturity "0"] [accuracy "0"] [tag "application-multi"] [tag "language-multi"] [tag "platform-multi"] [tag "attack-reputation-scanner"] [tag "OWASP_CRS"] [tag "OWASP_CRS/AUTOMATION/SECURITY_SCANNER"] [tag "WASCTC/WASC-21"] [tag "OWASP_TOP_10/A7"] [tag "PCI/6.5.10"] [hostname "172.17.0.2"] [uri "/actuator/health"] [unique_id "174680540065.651298"] [ref "o12,5v62,21t:lowercase"], client: 172.203.233.130, server: ~sven-dev(-vic)?.nimbusvideo.com, request: "GET /actuator/health HTTP/1.1", host: "108.163.5.103"
2025-05-09T15:43:20.434312461Z 2025/05/09 15:43:20 [info] 576#576: *7 ModSecurity: Warning. Matched "Operator `Rx' with parameter `^[\d.:]+$' against variable `REQUEST_HEADERS:Host' (Value: `108.163.5.103' ) [file "/usr/local/owasp-modsecurity-crs/rules/REQUEST-920-PROTOCOL-ENFORCEMENT.conf"] [line "678"] [id "920350"] [rev ""] [msg "Host header is a numeric IP address"] [data "108.163.5.103"] [severity "4"] [ver "OWASP_CRS/3.2.0"] [maturity "0"] [accuracy "0"] [tag "application-multi"] [tag "language-multi"] [tag "platform-multi"] [tag "attack-protocol"] [tag "OWASP_CRS"] [tag "OWASP_CRS/PROTOCOL_VIOLATION/IP_HOST"] [tag "WASCTC/WASC-21"] [tag "OWASP_TOP_10/A7"] [tag "PCI/6.5.10"] [hostname "172.17.0.2"] [uri "/actuator/health"] [unique_id "174680540065.651298"] [ref "o0,13v36,13"], client: 172.203.233.130, server: ~sven-dev(-vic)?.nimbusvideo.com, request: "GET /actuator/health HTTP/1.1", host: "108.163.5.103"
2025-05-09T15:43:20.435236474Z 2025/05/09 15:43:20 [error] 576#576: *7 [client 172.203.233.130] ModSecurity: Access denied with code 403 (phase 2). Matched "Operator `Ge' with parameter `5' against variable `TX:ANOMALY_SCORE' (Value: `8' ) [file "/usr/local/owasp-modsecurity-crs/rules/REQUEST-949-BLOCKING-EVALUATION.conf"] [line "79"] [id "949110"] [rev ""] [msg "Inbound Anomaly Score Exceeded (Total Score: 8)"] [data ""] [severity "2"] [ver ""] [maturity "0"] [accuracy "0"] [tag "application-multi"] [tag "language-multi"] [tag "platform-multi"] [tag "attack-generic"] [hostname "172.17.0.2"] [uri "/actuator/health"] [unique_id "174680540065.651298"] [ref ""], client: 172.203.233.130, server: ~sven-dev(-vic)?.nimbusvideo.com, request: "GET /actuator/health HTTP/1.1", host: "108.163.5.103"
2025-05-09T15:43:20.435712963Z 172.203.233.130 - - [09/May/2025:15:43:20 +0000] "GET /actuator/health HTTP/1.1" 403 146 "-" "Mozilla/5.0 zgrab/0.x" "-" 
2025-05-09T15:43:25.487845647Z 2025/05/09 15:43:25 [info] 576#576: *7 client 172.203.233.130 closed keepalive connection
2025-05-09T15:43:34.810673424Z 172.17.0.1 - - [09/May/2025:15:43:34 +0000] "GET /api HTTP/1.1" 200 97 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:43:34.811471622Z 2025/05/09 15:43:34 [info] 576#576: *8 client 172.17.0.1 closed keepalive connection
2025-05-09T15:43:48.728363366Z 2025/05/09 15:43:48 [info] 576#576: *10 client closed connection while waiting for request, client: 10.100.16.200, server: 0.0.0.0:443
2025-05-09T15:43:50.035879729Z 2025/05/09 15:43:50 [info] 576#576: *9 ModSecurity: Warning. Matched "Operator `Rx' with parameter `^[\d.:]+$' against variable `REQUEST_HEADERS:Host' (Value: `108.163.5.103' ) [file "/usr/local/owasp-modsecurity-crs/rules/REQUEST-920-PROTOCOL-ENFORCEMENT.conf"] [line "678"] [id "920350"] [rev ""] [msg "Host header is a numeric IP address"] [data "108.163.5.103"] [severity "4"] [ver "OWASP_CRS/3.2.0"] [maturity "0"] [accuracy "0"] [tag "application-multi"] [tag "language-multi"] [tag "platform-multi"] [tag "attack-protocol"] [tag "OWASP_CRS"] [tag "OWASP_CRS/PROTOCOL_VIOLATION/IP_HOST"] [tag "WASCTC/WASC-21"] [tag "OWASP_TOP_10/A7"] [tag "PCI/6.5.10"] [hostname "172.17.0.2"] [uri "/"] [unique_id "174680543098.602791"] [ref "o0,13v21,13"], client: 206.168.34.86, server: ~sven-dev(-vic)?.nimbusvideo.com, request: "GET / HTTP/1.1", host: "108.163.5.103"
2025-05-09T15:43:50.562422727Z 206.168.34.86 - - [09/May/2025:15:43:50 +0000] "GET / HTTP/1.1" 400 61933 "-" "Mozilla/5.0 (compatible; CensysInspect/1.1; +https://about.censys.io/)" "-" 
2025-05-09T15:43:50.991100212Z 2025/05/09 15:43:50 [info] 576#576: *9 client 206.168.34.86 closed keepalive connection
2025-05-09T15:43:52.911635302Z 2025/05/09 15:43:52 [info] 576#576: *12 client sent invalid request while reading client request line, client: 206.168.34.86, server: ~sven-dev(-vic)?.nimbusvideo.com, request: "PRI * HTTP/2.0"
2025-05-09T15:43:53.613498678Z 2025/05/09 15:43:53 [info] 576#576: *12 recv() failed (104: Connection reset by peer) while reading client request line, client: 206.168.34.86, server: ~sven-dev(-vic)?.nimbusvideo.com, request: "PRI * HTTP/2.0"
2025-05-09T15:43:53.613587397Z 206.168.34.86 - - [09/May/2025:15:43:53 +0000] "PRI * HTTP/2.0" 400 150 "-" "-" "-" 
2025-05-09T15:44:01.975482390Z 2025/05/09 15:44:01 [info] 576#576: *13 SSL_do_handshake() failed (SSL: error:14209102:SSL routines:tls_early_post_process_client_hello:unsupported protocol) while SSL handshaking, client: 206.168.34.86, server: 0.0.0.0:443
2025-05-09T15:44:04.811016099Z 172.17.0.1 - - [09/May/2025:15:44:04 +0000] "GET /api HTTP/1.1" 200 99 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:44:04.811382843Z 2025/05/09 15:44:04 [info] 576#576: *15 client 172.17.0.1 closed keepalive connection
2025-05-09T15:44:04.821858905Z 2025/05/09 15:44:04 [info] 576#576: *14 SSL_do_handshake() failed (SSL: error:14209102:SSL routines:tls_early_post_process_client_hello:unsupported protocol) while SSL handshaking, client: 206.168.34.86, server: 0.0.0.0:443
2025-05-09T15:44:07.043810620Z 2025/05/09 15:44:07 [info] 576#576: *16 SSL_do_handshake() failed (SSL: error:1420918C:SSL routines:tls_early_post_process_client_hello:version too low) while SSL handshaking, client: 206.168.34.86, server: 0.0.0.0:443
2025-05-09T15:44:22.555780072Z 2025/05/09 15:44:22 [info] 576#576: *17 ModSecurity: Warning. Matched "Operator `Rx' with parameter `^[\d.:]+$' against variable `REQUEST_HEADERS:Host' (Value: `108.163.5.103:443' ) [file "/usr/local/owasp-modsecurity-crs/rules/REQUEST-920-PROTOCOL-ENFORCEMENT.conf"] [line "678"] [id "920350"] [rev ""] [msg "Host header is a numeric IP address"] [data "108.163.5.103:443"] [severity "4"] [ver "OWASP_CRS/3.2.0"] [maturity "0"] [accuracy "0"] [tag "application-multi"] [tag "language-multi"] [tag "platform-multi"] [tag "attack-protocol"] [tag "OWASP_CRS"] [tag "OWASP_CRS/PROTOCOL_VIOLATION/IP_HOST"] [tag "WASCTC/WASC-21"] [tag "OWASP_TOP_10/A7"] [tag "PCI/6.5.10"] [hostname "172.17.0.2"] [uri "/"] [unique_id "174680546214.548008"] [ref "o0,17v21,17"], client: 206.168.34.86, server: ~sven-dev(-vic)?.nimbusvideo.com, request: "GET / HTTP/1.1", host: "108.163.5.103:443"
2025-05-09T15:44:23.119957575Z 206.168.34.86 - - [09/May/2025:15:44:23 +0000] "GET / HTTP/1.1" 400 61934 "-" "Mozilla/5.0 (compatible; CensysInspect/1.1; +https://about.censys.io/)" "-" 
2025-05-09T15:44:24.730559324Z 2025/05/09 15:44:24 [info] 576#576: *17 client 206.168.34.86 closed keepalive connection
2025-05-09T15:44:34.812156490Z 2025/05/09 15:44:34 [info] 576#576: *19 client 172.17.0.1 closed keepalive connection
2025-05-09T15:44:34.812180412Z 172.17.0.1 - - [09/May/2025:15:44:34 +0000] "GET /api HTTP/1.1" 200 100 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:45:04.811201455Z 172.17.0.1 - - [09/May/2025:15:45:04 +0000] "GET /api HTTP/1.1" 200 100 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:45:04.812063976Z 2025/05/09 15:45:04 [info] 576#576: *20 client 172.17.0.1 closed keepalive connection
2025-05-09T15:45:34.812154256Z 172.17.0.1 - - [09/May/2025:15:45:34 +0000] "GET /api HTTP/1.1" 200 100 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:45:34.812944951Z 2025/05/09 15:45:34 [info] 576#576: *21 client 172.17.0.1 closed keepalive connection
2025-05-09T15:46:04.813117817Z 172.17.0.1 - - [09/May/2025:15:46:04 +0000] "GET /api HTTP/1.1" 200 100 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:46:04.813543520Z 2025/05/09 15:46:04 [info] 576#576: *22 client 172.17.0.1 closed keepalive connection
2025-05-09T15:46:34.814794804Z 2025/05/09 15:46:34 [info] 576#576: *23 client 172.17.0.1 closed keepalive connection
2025-05-09T15:46:34.814824882Z 172.17.0.1 - - [09/May/2025:15:46:34 +0000] "GET /api HTTP/1.1" 200 100 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:47:04.814884863Z 172.17.0.1 - - [09/May/2025:15:47:04 +0000] "GET /api HTTP/1.1" 200 100 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:47:04.815650488Z 2025/05/09 15:47:04 [info] 576#576: *24 client 172.17.0.1 closed keepalive connection
2025-05-09T15:47:10.614837085Z 10.100.16.200 - - [09/May/2025:15:47:10 +0000] "GET / HTTP/1.1" 302 0 "-" "check_http/v2.4.12 (nagios-plugins 2.4.12)" "-" 
2025-05-09T15:47:34.816696415Z 172.17.0.1 - - [09/May/2025:15:47:34 +0000] "GET /api HTTP/1.1" 200 100 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:47:34.816702923Z 2025/05/09 15:47:34 [info] 576#576: *27 client 172.17.0.1 closed keepalive connection
2025-05-09T15:47:58.077681792Z 2025/05/09 15:47:58 [info] 576#576: *28 client closed connection while waiting for request, client: 10.100.16.200, server: 0.0.0.0:443
2025-05-09T15:48:04.815930258Z 172.17.0.1 - - [09/May/2025:15:48:04 +0000] "GET /api HTTP/1.1" 200 100 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:48:04.816671251Z 2025/05/09 15:48:04 [info] 576#576: *29 client 172.17.0.1 closed keepalive connection
2025-05-09T15:48:34.816417340Z 172.17.0.1 - - [09/May/2025:15:48:34 +0000] "GET /api HTTP/1.1" 200 100 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:48:34.817077088Z 2025/05/09 15:48:34 [info] 576#576: *30 client 172.17.0.1 closed keepalive connection
2025-05-09T15:48:46.717267994Z 2025/05/09 15:48:46 [info] 576#576: *31 client closed connection while waiting for request, client: 10.100.16.200, server: 0.0.0.0:443
2025-05-09T15:49:04.816641983Z 172.17.0.1 - - [09/May/2025:15:49:04 +0000] "GET /api HTTP/1.1" 200 100 "-" "Go-http-client/1.1" "-" 
2025-05-09T15:49:04.816985151Z 2025/05/09 15:49:04 [info] 576#576: *32 client 172.17.0.1 closed keepalive connection

### Assistant (2025-05-09 11:53:34)

**Why the call‑control requests are failing**

| Timestamp | Client IP | Request | Status | What it tells us |
|-----------|-----------|---------|--------|------------------|
| 15:43:08 Z | 172.16.102.139 | **POST /freeswitch_api/fs/in/pstn/park** | **404 (0 bytes)** | NGINX itself could not find a matching *location {}* block, so the request never reached FreeSWITCH. |
| 15:43:19 Z | 172.16.102.139 | **PUT /freeswitch_api/channel/hangup** | **404 (74 bytes)** | This one *did* hit an upstream, but the upstream answered 404—usually a sign that the URI was rewritten incorrectly on the way through. |
| 15:43:20 Z | 172.203.233.130 | **GET /actuator/health** | **403** | Blocked by ModSecurity rule‑set (anomaly score 8). Unrelated to call signalling. |

So the **phone‑call stack is not being blocked by ModSecurity**; it is simply never reaching the FreeSWITCH REST interface (or is reaching it with the wrong URI).  
The pattern is classic:

1. **First request (park)** – NGINX returns 404 immediately ⇒ no *location* matches `/freeswitch_api/…`.  
2. **Second request (hang‑up)** – a *location* does match, but the way `proxy_pass` is written causes the prefix `/freeswitch_api/` to be forwarded as well, and the FreeSWITCH REST app does not know that path, so it answers 404.

---

### Quick confirmation

```bash
# From inside the docker net‑namespace:
curl -i http://localhost/freeswitch_api/fs/in/pstn/park
# → 404 immediately     (NGINX)
curl -i http://freeswitch-backend:8080/fs/in/pstn/park
# → 200/201 (OK)        (FreeSWITCH)
```

If that second curl succeeds you have exactly the situation described above.

---

## How to fix it

> Adjust the `location` block that proxies `/freeswitch_api/` so that **(a)** it actually matches, and **(b)** it strips the prefix before forwarding.

```nginx
# inside the server { } that handles the container’s traffic
# (replace freeswitch_backend with your upstream or "127.0.0.1:8080")
upstream freeswitch_backend {
    server freeswitch:8080;   # or your real target
    keepalive 32;
}

# 1. Make sure the prefix is recognised
location ^~ /freeswitch_api/ {

    # 2. Strip the prefix – the trailing slash on proxy_pass is essential!
    #        ────────────┐
    proxy_pass http://freeswitch_backend/;
    #                     ↑ keeps only /fs/… in the request line

    # 3. Usual proxy hygiene
    proxy_set_header Host              $host;
    proxy_set_header X-Real-IP         $remote_addr;
    proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
    proxy_http_version 1.1;
    proxy_set_header Connection        "";
}
```

Why the trailing slash matters
------------------------------
* `proxy_pass http://freeswitch_backend/;` &nbsp;→&nbsp; **/freeswitch_api/** is removed before forwarding.  
* `proxy_pass http://freeswitch_backend;` &nbsp;&nbsp;&nbsp;→&nbsp; **/freeswitch_api/** is preserved, producing the wrong URI upstream and the 404 you saw.

---

## Optional: keep ModSecurity out of the signalling path

Although ModSecurity was *not* the culprit here, you may prefer to spare your internal call‑signalling traffic from CRS scoring:

```nginx
# immediately above the proxy_pass in the same location
ModSecurityEnabled        off;                # for NGINX‑modsec v1
# OR, for v3:
modsecurity_rules 'SecRuleEngine Off';
```

or a finer rule‑tweak:

```nginx
# e.g. disable rule 920350 (numeric Host header) just for this location
modsecurity_rules '
    SecRuleRemoveById 920350
';
```

---

### After reloading NGINX (`docker exec <container> nginx -s reload`)

1. Retry the call flow. The POST/PUT requests should now return 200/202 from FreeSWITCH.
2. Verify no new 404s appear in the access‑log for `/freeswitch_api/…`.
3. If you *still* get upstream 404s, inspect the exact path FreeSWITCH expects and adjust the strip prefix (sometimes the backend really does want the prefix—then drop the trailing slash on `proxy_pass` instead).

---

## One‑minute checklist

| Step | Action |
|------|--------|
| ① | Add **location ^~ /freeswitch_api/** as shown. |
| ② | Decide whether to keep or strip the prefix (use or omit `/` on `proxy_pass`). |
| ③ | Reload NGINX and confirm calls succeed. |
| ④ | Optionally disable CRS rules or the entire engine for this internal path. |

Once those two 404s disappear, the call‑control API will reach your FreeSWITCH servers and calls should complete again.

