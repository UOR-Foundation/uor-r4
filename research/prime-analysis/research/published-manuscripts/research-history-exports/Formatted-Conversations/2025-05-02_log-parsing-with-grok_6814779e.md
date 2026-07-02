# Log parsing with Grok

- **Conversation ID:** `6814779e-a43c-8001-b8f8-824d8c0c99b7`
- **Date:** 2025-05-02

---

### User (2025-05-02 03:43:26)

write me a grok or a regex parsing rule that can parse this kind of example log for New Relic to parse my logs of these kind:

07:21:20,024101012712,TRAFFIC,start,2817,2025/05/02 07:21:20,205.221.125.163,108.163.3.116,205.221.125.163,172.16.101.216,outside access to cafex media brokers,,,rtp_custom,vsys1,outside,dmz,ethernet1/13,ethernet1/17,syslog_traffic,2025/05/02 07:21:20,272624,1,42838,16001,42838,16001,0x404000,udp,allow,142,142,0,1,2025/05/02 07:21:21,0,any,,7491468824485099958,0x0,United States,United States,,1,0,n/a,0,0,0,0,,da6panfw02-3410,from-policy,,,0,,0,,N/A,0,0,0,0,2db7d029-26ee-447e-93f0-9d551fd04d5a,0,0,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,2025-05-02T07:21:20.797+00:00,,,photo-video,media,client-server,1,has-known-vulnerability,,rtp_custom,no,no,0,NonProxyTraffic,

### Assistant (2025-05-02 03:43:31)

Here are two ways you can pull out the key pieces of that comma-delimited firewall log.  You can either use a Logstash-style Grok pattern, or a single‐regex parsing rule for New Relic with named capture groups.  In both cases you’ll probably want to tweak the field names to suit exactly what you care about in your dashboards, but this will get you started.

---

### 1) Grok (Logstash) example

```conf
filter {
  grok {
    match => {
      "message" => [
        # — first 10 fields explicitly …
        "%{TIME:log_time},%{INT:session_id},%{WORD:category},%{WORD:action},%{INT:duration},%{YEAR}/%{MONTHNUM}/%{MONTHDAY} %{TIME:start_timestamp},%{IP:src_ip},%{IP:dest_ip},%{IP:src_nat_ip},%{IP:dest_nat_ip},
        # — free-form message (will capture “outside access to cafex media brokers”)
        %{DATA:description},
        # — three empty placeholders
        ,%{DATA:empty1},%{DATA:empty2},
        # — then a few more named fields, and finally everything else into “rest”
        %{DATA:application},%{DATA:vsys},%{DATA:from_zone},%{DATA:to_zone},%{DATA:ingress_interface},%{DATA:egress_interface},%{DATA:log_type},%{YEAR}/%{MONTHNUM}/%{MONTHDAY} %{TIME:syslog_time},%{INT:bytes_in},%{INT:packets},%{INT:bytes_out},%{GREEDYDATA:rest}"
      ]
    }
  }
}
```

**How it works**  
- `%{TIME:log_time}` → `"07:21:20"`  
- `%{INT:session_id}` → `"024101012712"`  
- `%{WORD:category}` → `"TRAFFIC"`  
- `%{WORD:action}` → `"start"`  
- `%{INT:duration}` → `"2817"`  
- `%{YEAR}/%{MONTHNUM}/%{MONTHDAY} %{TIME:start_timestamp}` → `"2025/05/02 07:21:20"`  
- `%{IP:src_ip}` → `"205.221.125.163"`  
- … and so on.  
- `%{DATA:description}` (non-comma text) picks up `outside access to cafex media brokers`  
- the three `,%{DATA:emptyX}` consume the empty `,, ,` slots,  
- `%{DATA:application}` → `rtp_custom`  
- … and `%{GREEDYDATA:rest}` grabs everything left.

---

### 2) New Relic “Regex” parsing rule

In New Relic you can define a “Regular expression” parser and drop in something like this:

```regex
^
(?<time>\d{2}:\d{2}:\d{2}),
(?<session_id>\d+),
(?<category>[^,]+),
(?<action>[^,]+),
(?<duration>\d+),
(?<start_timestamp>\d{4}\/\d{2}\/\d{2}\s\d{2}:\d{2}:\d{2}),
(?<src_ip>(?:\d{1,3}\.){3}\d{1,3}),
(?<dest_ip>(?:\d{1,3}\.){3}\d{1,3}),
(?<src_nat_ip>(?:\d{1,3}\.){3}\d{1,3}),
(?<dest_nat_ip>(?:\d{1,3}\.){3}\d{1,3}),
(?<description>[^,]*),
(?<empty1>),
(?<empty2>),
(?<application>[^,]*),
(?<vsys>[^,]*),
(?<from_zone>[^,]*),
(?<to_zone>[^,]*),
(?<ingress_interface>[^,]*),
(?<egress_interface>[^,]*),
(?<log_type>[^,]*),
(?<syslog_time>\d{4}\/\d{2}\/\d{2}\s\d{2}:\d{2}:\d{2}),
(?<bytes_in>\d+),
(?<packets>\d+),
(?<bytes_out>\d+),
(?<rest>.*)
$
```

- **Use**: In New Relic Logs → **Log parsing rules** → **Add a rule** → choose **Regex** → paste the above.  
- This will give you fields like `time`, `session_id`, `src_ip`, `description`, and then dump the remainder of the line into `rest` for any extra columns you want to parse later.

---

#### Tips for extending

- If you need _every_ column, just keep adding `,(?<fieldX>[^,]*)` for each comma-separated slot, or switch your final capture to `(?<rest>.*)` and do a second pass on `rest`.  
- If your date stamps ever change format (e.g. ISO 8601), swap in `%{TIMESTAMP_ISO8601:…}` or `\d{4}-\d{2}-\d{2}T…` as needed.  
- Grok lets you reuse patterns (`%{IP}` / `%{WORD}` / etc.); the pure-regex approach needs the full `(?:\d{1,3}\.){3}\d{1,3}` style.

Feel free to tweak the field names and positions to suit exactly what you want to chart in New Relic.

### User (2025-05-02 04:09:57)

please write me a log parsing pattern in Grok for the following log example for new relic log ingestion parsing:

stderr�Ù���esl_command.py (102) ERROR: ESL Command failure: URL: 'https://sven-pd-dc4-active.stratusvideo.com/freeswitch_api/fs/in/pstn/sessions' || DATA: {'conference_name': 'pstnin_18882537761-1_c2bcb707-0fc9-48c8-80f2-67443da1bc7a@dc4pdfs04.stratusvideo.com'} || SELF: {'sven_url': 'https://sven-pd-dc4-active.stratusvideo.com', 'session': <requests.sessions.Session object at 0x7f050308fd60>, 'esl_conn': <ESL.ESLconnection; proxy of <Swig Object of type 'ESLconnection *' at 0x7f050308fe40> >, 'event': <ESL.ESLevent; proxy of <Swig Object of type 'ESLevent *' at 0x7f0502685270> >, 'verify': True} || RESULT: {'_content': b'', '_content_consumed': True, '_next': None, 'status_code': 200, 'headers': {'Server': 'nginx', 'Date': 'Fri, 02 May 2025 07:24:29 GMT', 'Content-Length': '0', 'Connection': 'keep-alive', 'vary': 'Accept, Cookie', 'allow': 'POST, DELET || STATUS: 200 | ()p

### Assistant (2025-05-02 04:09:58)

Here’s a single Grok pattern you can drop into New Relic’s Log Parsing Rules (choose “Grok” and paste this in).  It will pull out the script, line number, log-level, URL, DATA, SELF, RESULT and STATUS fields, leaving the rest in `message`:

```
%{GREEDYDATA:__ignore__}(?<script>[^ ]+) \(%{NUMBER:line_number}\) %{LOGLEVEL:log_level}: ESL Command failure: URL: '%{URI:url}' \|\| DATA: %{DATA:data} \|\| SELF: %{DATA:self} \|\| RESULT: %{DATA:result} \|\| STATUS: %{NUMBER:status_code} \| %{GREEDYDATA:message}
```

**Field breakdown**  
- `script`         → `esl_command.py`  
- `line_number`    → `102`  
- `log_level`      → `ERROR`  
- `url`            → `https://sven-pd-dc4-active.stratusvideo.com/freeswitch_api/fs/in/pstn/sessions`  
- `data`           → `{'conference_name': 'pstnin_18882537761-1_c2bcb707-0fc9-48c8-80f2-67443da1bc7a@dc4pdfs04.stratusvideo.com'}`  
- `self`           → `{'sven_url': 'https://sven-pd-dc4-active.stratusvideo.com', 'session': <requests.sessions.Session object at 0x7f050308fd60>, ...}`  
- `result`         → `{'_content': b'', '_content_consumed': True, ...}`  
- `status_code`    → `200`  
- `message`        → whatever remains after the final `| ` (e.g. `()` plus any trailing junk)

The leading `%{GREEDYDATA:__ignore__}` slurps up any control characters or prefixes you don’t care about. You can then use each of these fields in your New Relic dashboards and alerts.

### User (2025-05-02 04:12:06)

Will the same parsing patter work for this example log as well?

stderr���������esl_command.py (102) ERROR: ESL Command failure: URL: 'https://sven-pd-dc4-active.stratusvideo.com/freeswitch_api/fs/in/video/sessions' || DATA: {'conference_name': 'zin_7002034894_41a3533a-17ee-48dc-9843-7ce45b0550ad@dc4pdfs04.stratusvideo.com'} || SELF: {'sven_url': 'https://sven-pd-dc4-active.stratusvideo.com', 'session': <requests.sessions.Session object at 0x7f050308fd60>, 'esl_conn': <ESL.ESLconnection; proxy of <Swig Object of type 'ESLconnection *' at 0x7f050308fe40> >, 'event': <ESL.ESLevent; proxy of <Swig Object of type 'ESLevent *' at 0x7f0502685540> >, 'verify': True} || RESULT: {'_content': b'<html>\r\n<head><title>503 Service Temporarily Unavailable</title></head>\r\n<body>\r\n<center><h1>503 Service Temporarily Unavailable</h1></center>\r\n<hr><center>nginx</center>\r\n</body>\r\n</html>\r\n', '_content_consumed': True, ' || STATUS: 503 | ()zx

### Assistant (2025-05-02 04:12:06)

Yes – because the second error line has the exact same “URL: … || DATA: … || SELF: … || RESULT: … || STATUS: …” structure, the pattern I gave you will happily pick up:

- the new `/video/sessions` URL  
- the updated DATA dict  
- the SELF dict  
- the full HTML body in RESULT  
- and the 503 status  

You don’t even need to change anything.

---

### But, one small tweak for extra robustness

If you want to be absolutely sure you grab *all* of the embedded commas in the RESULT (or DATA) block (even if it grows or adds more fields), you can switch those to `GREEDYDATA` instead of `DATA`. For example:

```grok
%{GREEDYDATA:__ignore__}(?<script>\S+) \(%{NUMBER:line_number}\) %{LOGLEVEL:log_level}: ESL Command failure: URL: '%{URI:url}' \|\| DATA: %{GREEDYDATA:data} \|\| SELF: %{GREEDYDATA:self} \|\| RESULT: %{GREEDYDATA:result} \|\| STATUS: %{NUMBER:status_code} \| %{GREEDYDATA:message}
```

- `%{GREEDYDATA:data}` will suck up everything after `DATA:` up to the next ` || SELF:`  
- `%{GREEDYDATA:result}` will suck up the entire HTML payload (with its commas/newlines) right up to ` || STATUS:`  

This one‐pattern covers *both* your PSTN example **and** your VIDEO example (and any future variants that keep the same separators).

### User (2025-05-02 04:18:55)

Please write me another grok parsing pattern that will parse the following example log:

2025-05-02 04:11:56,002 DEBUG [com.alicecallsbob.porth.gateway.configuration.AbstractGatewayDataStore] (EJB default - 7) | Processing: {"bannedCodecs":["Red","G722","SILK","g7221","ilbc","RTX","flexfec-03","CN","MP4A-LATM","ULPFEC","isac","X-H264UC"],"maximumVideoResolution":{"height":480,"width":640},"defaultVideoResolution":{"height":480,"width":640},"audioPrioritisedCodecs":[],"videoPrioritisedCodecs":[],"outboundSipServerAddresses":["sip:p10st2sipproxy01.nimbusvideo.com:5050"],"sipServerTimeout":3000,"pingInterval":30000,"deadLinkPingInterval":5000,"rewriteOutboundSipUris":false,"registrarSipUri":"","registrationExpires":"1800","sipSessionExpiry":10800,"minSipSessionExpiry":400,"maxMessageQueueSize":1000,"clientPingPollPeriod":60,"missedPingsBeforeTimeout":5,"fastPictureUpdatePollPeriod":0,"mediaTimeoutPollPeriod":15,"maxLoadLimit":0,"rtpTransportType":"BOTH","webApplicationKeys":["sven-LNZd76Bv1IsXq7Urhek"],"serverGroup":null,"aedDefaultTimeout":0,"loadBalancerHttpPort":8080,"framesPerSecond":30,"scalingMode":"NONE","defaultQValue":1.0,"responseCodesToRetry":[],"defaultExpiresTime":21600,"minimumExpiresTime":5,"acceptingExternalRegistrations":true,"callLogExpiry":60,"statisticLogLevel":"OFF","performanceLogEnabled":true,"performanceLogExpiry":60,"performanceLogSamplePeriod":60,"adaptiveBitrateEnabled":true,"adaptiveBitrateInitial":512,"adaptiveBitrateFloor":348,"adaptiveBitrateCeiling":512,"fixedBitrateAudio":null,"fixedBitrateVideo":null,"maxConcurrentSessions":5000,"iePluginUrl":"/ie/WebRTC-Plugin.msi","iePluginUrlGatewayRelative":true,"iePluginVersion":"3.5.5.1","iePluginMinVersion":"3.3.12.0","pictureLossMechanism":"PLI","restrictMediaToSdpPorts":true,"dynamicCodecsEnabled":false,"usingExternalRegistrar":false,"rtpProxyList":[{"controlHost":"172.16.102.119","controlPort":8092,"controlSecure":false,"rtpProxyConfigurationDetails":{"externalPortsSpecification":[{"browserSourcePattern":{"notation":"*","prefixSize":0,"wildcard":true},"browserPorts":[{"localPort":{"address":"172.16.102.119","port":16004},"publicPorts":[{"address":"108.163.5.119","port":16004}]},{"localPort":{"address":"172.16.102.119","port":16003},"publicPorts":[{"address":"108.163.5.119","port":16003}]},{"localPort":{"address":"172.16.102.119","port":16002},"publicPorts":[{"address":"108.163.5.119","port":16002}]},{"localPort":{"address":"172.16.102.119","port":16001},"publicPorts":[{"address":"108.163.5.119","port":16001}]},{"localPort":{"address":"172.16.102.119","port":16000},"publicPorts":[{"address":"108.163.5.119","port":16000}]}]}],"externalMowsPortsSpecification":[],"internalPortsSpecification":[{"cidr":{"notation":"172.16.102.119/32","prefixSize":32,"wildcard":false},"lowerPort":17000,"upperPort":17599}],"maxBufferSize":500,"rtpProxyLimits":{"idleRouteTimeout":30,"packetSizeLimit":1500,"throughputRateLimit":1000,"maxAudioOnlyCallLimit":0,"maxAudioVideoCallLimit":0},"networkConnectivityGroups":[]}}],"gwMbSdpRequestTimeoutMSecs":0}

### Assistant (2025-05-02 04:18:56)

Use this Grok pattern in New Relic (Logs → Log parsing rules → Grok) to pull out the timestamp, level, logger/class, thread, operation, and the full JSON blob:

```
^%{TIMESTAMP_ISO8601:timestamp} %{LOGLEVEL:log_level} \[%{DATA:logger}\] \(%{DATA:thread}\) \| %{WORD:operation}: %{GREEDYDATA:config_json}$
```

**Field breakdown**  
- `timestamp`   → `2025-05-02 04:11:56,002`  
- `log_level`   → `DEBUG`  
- `logger`      → `com.alicecallsbob.porth.gateway.configuration.AbstractGatewayDataStore`  
- `thread`      → `EJB default - 7`  
- `operation`   → `Processing`  
- `config_json` → the entire `{"bannedCodecs":[…],"gwMbSdpRequestTimeoutMSecs":0}` object  

If you need to further parse individual JSON fields you can either:

1. **Let New Relic’s automatic JSON parsing** pick up keys (if you mark the field as JSON), or  
2. Chain another Grok/JSON parser on `config_json`.  

This single pattern will reliably capture the full payload, even as your JSON grows or changes.

### User (2025-05-02 05:09:05)

please write me a log ingestion parsing rule using GROK for the following example log:

08:39:34,024101012660,SYSTEM,auth,2817,2025/05/02 08:39:35,,auth-fail,vpn_saml,0,0,general,medium,"failed authentication for user 'mbrothers'. Reason: Internal error, e.g. network connection, DNS failure or remote server down. auth profile 'vpn_saml', vsys 'vsys1', From: 173.239.240.153.",7494079381552717750,0x0,0,0,0,0,,dc4panfw02-3410,0,0,2025-05-02T08:39:35.174+00:00

### Assistant (2025-05-02 05:09:06)

Here’s a single Grok pattern you can paste into New Relic’s “Grok” log-parsing rule. It will pull out all of the comma-delimited fields (including the quoted error message) into named attributes:

```grok
^%{TIME:log_time},%{NUMBER:session_id},%{WORD:category},%{WORD:auth_type},%{NUMBER:duration},%{YEAR}/%{MONTHNUM}/%{MONTHDAY} %{TIME:start_timestamp},%{DATA:empty1},%{WORD:result},%{WORD:auth_profile},%{NUMBER:code1},%{NUMBER:code2},%{WORD:policy},%{WORD:risk},"%{DATA:message}",%{NUMBER:event_id},%{DATA:flags_hex},%{NUMBER:bytes_in},%{NUMBER:bytes_out},%{NUMBER:packets},%{NUMBER:flows},%{DATA:empty2},%{DATA:device},%{NUMBER:field23},%{NUMBER:field24},%{TIMESTAMP_ISO8601:iso_ts}$
```

#### Field breakdown

| Grok field              | Captures                                                               |
|-------------------------|------------------------------------------------------------------------|
| `log_time`              | `08:39:34`                                                              |
| `session_id`            | `024101012660`                                                          |
| `category`              | `SYSTEM`                                                                |
| `auth_type`             | `auth`                                                                  |
| `duration`              | `2817`                                                                  |
| `start_timestamp`       | `2025/05/02 08:39:35`                                                   |
| `empty1`                | (the blank field after the timestamp)                                   |
| `result`                | `auth-fail`                                                             |
| `auth_profile`          | `vpn_saml`                                                              |
| `code1`                 | `0`                                                                     |
| `code2`                 | `0`                                                                     |
| `policy`                | `general`                                                               |
| `risk`                  | `medium`                                                                |
| `message`               | `failed authentication for user 'mbrothers'. Reason: Internal error, e.g. network connection, DNS failure or remote server down. auth profile 'vpn_saml', vsys 'vsys1', From: 173.239.240.153.` |
| `event_id`              | `7494079381552717750`                                                   |
| `flags_hex`             | `0x0`                                                                   |
| `bytes_in`              | `0`                                                                     |
| `bytes_out`             | `0`                                                                     |
| `packets`               | `0`                                                                     |
| `flows`                 | `0`                                                                     |
| `empty2`                | (the blank field before the device)                                      |
| `device`                | `dc4panfw02-3410`                                                       |
| `field23`               | `0`                                                                     |
| `field24`               | `0`                                                                     |
| `iso_ts`                | `2025-05-02T08:39:35.174+00:00`                                         |

**How to use:**  
1. In New Relic, go to **Logs → Log parsing rules**.  
2. Click **Add a rule**, choose **Grok**.  
3. Paste the pattern above and save.  

Now you’ll have all of those fields available for dashboards, alerts, and searches.

