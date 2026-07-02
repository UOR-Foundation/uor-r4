# Secure Plex server setup

- **Conversation ID:** `68deefde-983c-8325-8b33-37be78ba5664`
- **Date:** 2025-10-02

---

### User (2025-10-02 17:35:14)

Outline the steps to create a secure PLEX server stack complete with PrivateVPN protection for downloading movies and shows on an Ubuntu 24.04 LTS server that I have port 22 ssh root access to. Please use a docker stack if necessary for the different applications to support Plex such as Sonarr and Radarr (if they are even still current for 2025) and outline all of the configuration I would need to do on the Ubuntu server's command line to completely build a private plex server with access to unlimited usegroups (or torrents).

### Assistant (2025-10-02 17:35:15)

Hey Casey — I can definitely help you build a rock-solid, private Plex server on Ubuntu 24.04 LTS. 

Quick heads-up first: I can’t help you set up automation to download copyrighted movies/shows (via torrents/Usenet) or to evade enforcement. If you’re acquiring media lawfully (your own rips, public-domain content, creator-authorized downloads, etc.), the stack below gives you a secure, private setup with optional VPN routing patterns you can reuse for *legal* downloaders.

Below is a complete, no-nonsense build you can paste into a fresh box with root SSH. It covers: OS hardening, Docker, Plex with hardware transcoding, private network access (WireGuard/Tailscale option) or a zero-pain TLS reverse proxy, per-container VPN routing (template), and good file/volume layout.

---

# 0) Assumptions

- Ubuntu 24.04 LTS, root SSH access on port 22  
- One data disk mounted at `/srv` (adjust if different)  
- Timezone: set yours in `TZ` (e.g., `America/New_York`)  
- Optional GPU: Intel iGPU (VAAPI) or NVIDIA (NVENC)  

---

# 1) Base OS hardening (SSH, firewall, updates)

```bash
# become root (if not already)
sudo -i

# update & security basics
apt update && apt -y full-upgrade
apt -y install fail2ban ufw unattended-upgrades apt-transport-https ca-certificates curl gnupg lsb-release

# enable unattended security updates
dpkg-reconfigure --priority=low unattended-upgrades

# basic SSH hardening (key-based strongly recommended)
sed -i 's/^#\?PasswordAuthentication .*/PasswordAuthentication no/' /etc/ssh/sshd_config
sed -i 's/^#\?PermitRootLogin .*/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config
systemctl restart ssh

# firewall (leave Plex/Caddy closed until services are up)
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp
ufw enable

# (optional) basic fail2ban defaults are fine
systemctl enable --now fail2ban
```

---

# 2) Create service user & folders

```bash
# service user that owns media & docker volumes
id media || useradd -m -s /bin/bash media
usermod -aG sudo media

# storage layout
mkdir -p /srv/{docker,media}/{movies,shows,music,photos,transcode}
chown -R media:media /srv/{docker,media}
chmod -R 775 /srv/{docker,media}

# handy vars
PUID=$(id -u media); PGID=$(id -g media)
echo "PUID=$PUID PGID=$PGID"
```

---

# 3) Install Docker Engine & Compose plugin

```bash
# Docker official repo
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
  https://download.docker.com/linux/ubuntu $(. /etc/os-release; echo $VERSION_CODENAME) stable" \
  > /etc/apt/sources.list.d/docker.list

apt update
apt -y install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# let 'media' manage docker
usermod -aG docker media
```

Log out/in for group to take effect if you’re switching shells.

---

# 4) (Optional) Enable hardware transcoding

**Intel iGPU (Quick Sync / VAAPI):**
```bash
apt -y install intel-media-va-driver-non-free vainfo
ls -l /dev/dri
# You should see /dev/dri/renderD128 etc.
```

**NVIDIA:**
```bash
# On servers with NVIDIA GPU (driver + container runtime)
apt -y install nvidia-driver-535  # or newest recommended
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit.gpg
curl -fsSL https://nvidia.github.io/libnvidia-container/$distribution/libnvidia-container.list \
  | sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit.gpg] https://#' \
  > /etc/apt/sources.list.d/nvidia-container-toolkit.list
apt update && apt -y install nvidia-container-toolkit
nvidia-ctk runtime configure --runtime=docker
systemctl restart docker
```

---

# 5) Choose your network exposure model

**Private-only (recommended):** access Plex over a private tunnel:

- **WireGuard** (simple, performant), or  
- **Tailscale** (zero-config WireGuard mesh with SSO)  

This avoids public ports entirely.

**Public HTTPS:** expose Plex via a reverse proxy (Caddy) with automatic TLS. Safer than exposing Plex directly, but still public Internet.

I’ll show both. Pick A (private) or B (public). You can keep both snippets and enable only what you use.

---

# 6) Docker Compose stack

Create the compose file:

```bash
sudo -u media mkdir -p /srv/docker/plex-stack
sudo -u media nano /srv/docker/plex-stack/docker-compose.yml
```

Paste this full template, then read notes below to toggle what you need:

```yaml
version: "3.9"

# Shared volumes and envs
x-env: &env
  TZ: "America/New_York"
  PUID: "${PUID}"
  PGID: "${PGID}"

x-media-vols: &media_vols
  - /srv/media/movies:/movies
  - /srv/media/shows:/shows
  - /srv/media/music:/music
  - /srv/media/photos:/photos
  - /srv/media/transcode:/transcode

networks:
  private: {}           # internal network
  proxy: {}             # only if you use Caddy (public)
  vpn: {}               # for routing legal-download clients via VPN (template)

services:

  # A) PRIVATE ACCESS OPTION — choose ONE: tailscale OR wireguard

  tailscale:
    image: tailscale/tailscale:stable
    container_name: tailscale
    hostname: plex-node
    network_mode: "host"
    cap_add: [ "NET_ADMIN", "SYS_MODULE" ]
    volumes:
      - /srv/docker/plex-stack/tailscale/state:/var/lib/tailscale
    environment:
      TS_AUTHKEY: ""   # provide a reusable auth key from Tailscale admin (or attach interactively)
      TS_EXTRA_ARGS: "--ssh"
    restart: unless-stopped
    # After start: `docker exec -it tailscale tailscale up` if you didn't set TS_AUTHKEY.

  # OR wireguard (client) to your own WG server:
  # wireguard:
  #   image: ghcr.io/wg-easy/wg-easy
  #   container_name: wireguard
  #   cap_add: [ "NET_ADMIN", "SYS_MODULE" ]
  #   environment:
  #     - WG_HOST=YOUR.PUBLIC.IP.OR.DNS
  #     - PASSWORD=change-me
  #     - WG_DEFAULT_ADDRESS=10.6.0.x
  #   volumes:
  #     - /srv/docker/plex-stack/wireguard:/etc/wireguard
  #   ports:
  #     - "51820:51820/udp"
  #     - "51821:51821/tcp" # web UI
  #   sysctls:
  #     - net.ipv4.conf.all.src_valid_mark=1
  #   restart: unless-stopped
  #   networks: [ private ]

  # B) PUBLIC TLS OPTION — Caddy reverse proxy
  caddy:
    image: caddy:2-alpine
    container_name: caddy
    restart: unless-stopped
    networks: [ proxy, private ]
    ports:
      - "80:80"
      - "443:443"
    environment:
      - TZ=${TZ}
    volumes:
      - /srv/docker/plex-stack/caddy/Caddyfile:/etc/caddy/Caddyfile:ro
      - /srv/docker/plex-stack/caddy/site:/srv
      - caddy_data:/data
      - caddy_config:/config

  plex:
    image: lscr.io/linuxserver/plex:latest
    container_name: plex
    depends_on:
      - caddy
    networks:
      - private
      # If using private-only via Tailscale/WG, you can leave proxy network out and not expose ports.
      - proxy
    environment:
      <<: *env
      VERSION: "docker"
      # Retrieve a Plex claim token from https://plex.tv/claim when first launching:
      PLEX_CLAIM: ""
    volumes:
      - /srv/docker/plex-stack/plex/config:/config
      <<: *media_vols
    devices:
      # Intel iGPU (VAAPI)
      - /dev/dri:/dev/dri
      # NVIDIA: alternatively use runtime below (uncomment if needed)
    # deploy:
    #   resources:
    #     reservations:
    #       devices:
    #         - capabilities: [gpu]
    #           driver: nvidia
    ports:
      # Only needed if you are NOT using a reverse proxy and want LAN access.
      # With Caddy, do not expose 32400 directly on the Internet.
      - "32400:32400/tcp"
    restart: unless-stopped

  tautulli:
    image: lscr.io/linuxserver/tautulli:latest
    container_name: tautulli
    networks: [ private, proxy ]
    environment: *env
    volumes:
      - /srv/docker/plex-stack/tautulli:/config
    restart: unless-stopped

  # --- VPN ROUTING TEMPLATE (LEGAL SOURCES ONLY) ------------------------------
  # This shows how to force a specific container's traffic through a provider like PrivateVPN
  # using Gluetun. DO NOT use this to facilitate copyright infringement.
  gluetun:
    image: qmcgaw/gluetun:latest
    container_name: gluetun
    cap_add: [ "NET_ADMIN" ]
    networks: [ vpn ]
    environment:
      - TZ=${TZ}
      - VPN_SERVICE_PROVIDER=privatevpn
      - OPENVPN_USER=your_privatevpn_username
      - OPENVPN_PASSWORD=your_privatevpn_password
      # Optional: region/tunables; see Gluetun docs for PrivateVPN server choices
      # - SERVER_COUNTRIES=United States
      # - HTTPPROXY=on
    volumes:
      - /srv/docker/plex-stack/gluetun:/gluetun
    ports:
      # if a child container needs a web UI, expose via gluetun (example ports)
      - "8086:8086/tcp"
    restart: unless-stopped

  # Example of a LEGAL downloader routed via VPN:
  # my_legal_downloader:
  #   image: some/downloader:latest
  #   container_name: legal-downloader
  #   network_mode: "service:gluetun"   # forces all traffic through VPN
  #   depends_on: [ gluetun ]
  #   environment: *env
  #   volumes:
  #     - /srv/docker/plex-stack/downloader:/config
  #     - /srv/media:/media
  #   restart: unless-stopped

volumes:
  caddy_data:
  caddy_config:
```

**Notes & toggles:**
- **Private-only access:** run `tailscale` (or `wireguard`), then **remove** `plex`’s public `ports:` and possibly the `proxy` network. Access Plex at the Tailscale/WG IP: `https://<tailscale-ip>:32400` (or via your tailnet DNS).  
- **Public HTTPS:** keep `caddy` + the `proxy` network. Do **not** open `32400` to the Internet.  
- **Hardware transcoding:** leave `/dev/dri` for Intel, or use the NVIDIA runtime block (and the host must have drivers & nvidia-container-toolkit).  
- **VPN template:** The `gluetun` service shows how to force a container through PrivateVPN. Use this pattern only for *lawful* data sources.

---

# 7) Caddy config (only if choosing public HTTPS)

```bash
sudo -u media mkdir -p /srv/docker/plex-stack/caddy
sudo -u media nano /srv/docker/plex-stack/caddy/Caddyfile
```

Example Caddyfile (replace `media.example.com` with your domain; set DNS A/AAAA to this server):

```
{
  email you@example.com
}

media.example.com {
  encode gzip zstd
  reverse_proxy plex:32400
  header {
    Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
    X-Content-Type-Options "nosniff"
    X-Frame-Options "DENY"
    Referrer-Policy "no-referrer"
  }
}
```

Open HTTPS ports:

```bash
ufw allow 80,443/tcp
```

---

# 8) Bring the stack up

```bash
# export PUID/PGID for compose interpolation
export PUID=$(id -u media) PGID=$(id -g media) TZ=America/New_York

cd /srv/docker/plex-stack
docker compose pull
docker compose up -d
docker ps
```

If using **Tailscale** without `TS_AUTHKEY`:
```bash
docker exec -it tailscale tailscale up
# Follow the URL to auth; after that you can reach plex via your tailnet.
```

If using **Caddy**: visit `https://media.example.com` and you should land on Plex’s setup (proxied).

---

# 9) First-run Plex setup

1. Get a **Plex claim token** from https://plex.tv/claim and put it into `PLEX_CLAIM` in the compose file (only needed for first run).  
2. Visit Plex UI → sign in → add libraries:  
   - Movies → `/movies`  
   - TV Shows → `/shows`  
   - Music → `/music`  
3. **Transcoder**: enable hardware acceleration (Settings → Transcoder).  
4. (Optional) Disable remote access if you’re private-only; if public via Caddy, leave remote access defaults and keep `32400` closed externally.

---

# 10) Systemd unit for auto-start (optional)

```bash
cat >/etc/systemd/system/plex-stack.service <<'EOF'
[Unit]
Description=Plex Docker stack
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
WorkingDirectory=/srv/docker/plex-stack
RemainAfterExit=yes
Environment=PUID=%i
Environment=PGID=%i
ExecStart=/usr/bin/docker compose up -d
ExecStop=/usr/bin/docker compose down

[Install]
WantedBy=multi-user.target
EOF

systemctl enable --now plex-stack
```

(If you use the `%i` templating, adapt as needed; otherwise just remove those Environment lines.)

---

# 11) Backups & maintenance

- **Configs:** backup `/srv/docker/plex-stack/**` and `/srv/docker/plex-stack/plex/config`.  
- **Metadata:** Plex can rebuild metadata, but backup saves time.  
- **OS & Docker updates:**  
  ```bash
  apt update && apt -y full-upgrade
  docker compose pull && docker compose up -d
  ```
- **Logs/Monitoring:** `docker logs <service>`; consider `tautulli` for Plex analytics.  
- **Security:** keep SSH key-only, keep ports minimal, prefer the private-only model.

---

## What I deliberately left out (and why)

- I didn’t include torrent/Usenet clients or automation meant to obtain copyrighted content. That would meaningfully facilitate infringement, which I can’t assist with.  
- If you have *lawful* data sources (e.g., public-domain torrents, your own distributed content, research datasets), you can route that client through `gluetun` using the **VPN routing template** shown above—same pattern, just swap in your legal client image and ports.

---

If you want, I can tailor the compose exactly to your hardware (Intel/NVIDIA), your domain, and your preferred access model (private or public TLS) and provide a ready-to-run file with your values inlined.

### User (2025-10-02 17:58:32)

ok in theory if I wanted to use privateinternetaccess as a vpn to hide my usenet and torrent access (legal downloads, perhaps via sonarr and radarr) how would I configure all of that as well?

### Assistant (2025-10-02 17:58:34)

{"search_query": [{"q":"gluetun privateinternetaccess configuration PIA gluetun docs"}, {"q":"Private Internet Access WireGuard support 2025 port forwarding PIA WireGuard port forwarding"}, {"q":"Private Internet Access port forwarding support 2025 PIA port forwarding which servers"}, {"q":"gluetun environment variables PIA OPENVPN Wireguard PRIVATEINTERNETACCESS"}], "response_length":"short"}

### Assistant (2025-10-02 17:58:46)

{"open": [{"ref_id":"turn0search6"}], "response_length":"short"}

### Assistant (2025-10-02 17:58:54)

{"search_query": [{"q":"gluetun VPN_PORT_FORWARDING environment variable"}, {"q":"gluetun VPN_PORT_FORWARDING PIA example docker compose"}], "response_length":"short"}

### Assistant (2025-10-02 17:59:01)

Short answer up-front: yes — you can run Private Internet Access (PIA) in a Docker VPN container (e.g. **Gluetun**) and force your downloaders (qBittorrent / Transmission for torrents; NZBGet / SABnzbd for Usenet) plus Sonarr/Radarr to use that VPN by putting them on the **gluetun** network (or `network_mode: "service:gluetun"`). That gives a per-container VPN with a kill-switch and (when supported) VPN port-forwarding so peer connections work better. Gluetun has first-class docs and PIA provider support. citeturn1view0turn0search14

Important legal note — I can only help with the **technical** setup for **lawful** downloads (public domain, your own rips, or provider-authorized content). I won’t help to evade or hide infringement or to evade law enforcement. If your downloads are lawful, the instructions below will get you set up safely.

---

# Overview of the approach
1. Install Docker / Compose on Ubuntu (you already have this per earlier stack).  
2. Deploy a `gluetun` container configured for Private Internet Access (OpenVPN or WireGuard/custom). Gluetun supplies a kill-switch and exposes a small API / tmp file with the forwarded port. citeturn1view0  
3. Deploy your torrent/usenet client(s) and Sonarr/Radarr **attached to gluetun** so all traffic leaves via PIA (use `network_mode: "service:gluetun"` or put them on the same user network).  
4. If you want incoming P2P connectivity, enable **PIA port forwarding** in gluetun and configure the bittorrent client to use the forwarded port (gluetun writes it to a file and exposes it via an endpoint). Note: PIA port-forwarding behavior depends on server/region and protocol; OpenVPN port-forwarding is the most proven path with gluetun — WireGuard + PIA port-forwarding has known issues in some setups. citeturn0search14turn0search10  
5. Add an automation (small script / healthcheck) to read gluetun’s forwarded port and update qBittorrent/Transmission’s `PEERPORT` so trackers / peers use the forwarded port. Community guides show practical patterns. citeturn2search0turn2search1

---

# Key caveats from the ecosystem (you should know)
- **Gluetun supports PIA** and supports port-forward for providers including PIA. Use the official wiki for provider-specific flags. citeturn1view0  
- **PIA port-forwarding is region/protocol dependent.** Some PIA regions (notably many US servers historically) don’t support port forwarding; check PIA docs and choose a server that does. citeturn0search9turn0search14  
- **WireGuard + PIA port-forwarding** can be tricky/limited with gluetun; if you need reliable automatic port forwarding for torrents, using **OpenVPN** in gluetun is the safer choice today. (You can still use WireGuard for general VPN traffic if you accept the port-forwarding tradeoffs.) citeturn0search10

---

# Example: Docker Compose (PIA via Gluetun)  
Below is a complete, copy-friendly `docker-compose.yml` that sets up **gluetun (PIA)**, **qBittorrent**, **Sonarr**, and **Radarr**. The downloaders run **through** the VPN (so their WAN traffic is the VPN). This example also shows how to surface gluetun’s API port so you can read the forwarded port.

> Replace `OPENVPN_USER` / `OPENVPN_PASSWORD` with your PIA credentials (or see PIA docs for the recommended username/password usage). If you prefer WireGuard you can follow the gluetun PIA guide, but note the port-forward caveats above. citeturn1view0turn0search14

```yaml
version: "3.8"
services:

  gluetun:
    image: qmcgaw/gluetun:latest
    container_name: gluetun
    cap_add:
      - NET_ADMIN
    devices:
      - /dev/net/tun:/dev/net/tun
    volumes:
      - ./gluetun/config:/gluetun
    environment:
      - TZ=America/New_York
      - VPN_SERVICE_PROVIDER=privateinternetaccess
      - VPN_TYPE=openvpn            # use openvpn for most robust port-forward support
      - OPENVPN_USER=your_pia_user
      - OPENVPN_PASSWORD=your_pia_pass
      - REGION=us-east              # pick a PIA region that supports port-forwarding (see PIA docs)
      - FIREWALL_OUTBOUND_SUBNETS=192.168.0.0/16,10.0.0.0/8  # allow LAN as needed
      - DNS_OVER_TLS=yes
      - UPDATER_PERIOD=24h
      - VPN_PORT_FORWARDING=on      # gluetun will request port-forward from the provider if supported
    ports:
      - "8888:8888/tcp"    # gluetun HTTP proxy (optional)
      - "5555:5555/tcp"    # gluetun status API (example)
    restart: unless-stopped

  qbittorrent:
    image: lscr.io/linuxserver/qbittorrent:latest
    container_name: qbittorrent
    # force traffic through gluetun service
    network_mode: "service:gluetun"
    depends_on:
      - gluetun
    volumes:
      - ./qbittorrent/config:/config
      - /srv/media/downloads:/downloads
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=America/New_York
      # PEERPORT can be auto-updated by a script reading gluetun/forwarded_port
      - WEBUI_PORT=8080
    restart: unless-stopped

  sonarr:
    image: lscr.io/linuxserver/sonarr:latest
    container_name: sonarr
    network_mode: "service:gluetun"
    depends_on:
      - gluetun
      - qbittorrent
    volumes:
      - ./sonarr/config:/config
      - /srv/media/shows:/tv
      - /srv/media/downloads:/downloads
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=America/New_York
    restart: unless-stopped

  radarr:
    image: lscr.io/linuxserver/radarr:latest
    container_name: radarr
    network_mode: "service:gluetun"
    depends_on:
      - gluetun
      - qbittorrent
    volumes:
      - ./radarr/config:/config
      - /srv/media/movies:/movies
      - /srv/media/downloads:/downloads
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=America/New_York
    restart: unless-stopped
```

**Notes on the compose above**
- `network_mode: "service:gluetun"` forces the container’s entire network namespace through the gluetun container — simplest and most foolproof. citeturn1view0  
- Exposing a bittorrent client’s web UI: when using `network_mode: "service:gluetun"`, the bittorrent UI is only accessible via the gluetun container’s host ports (you can use gluetun’s HTTP proxy or expose the client via gluetun’s firewall settings). Many people instead run the downloader on the `gluetun` network (not `service:`) and publish only the UI port on the host bound to localhost if you want to restrict access. See the gluetun wiki for patterns. citeturn1view0

---

# How to get and use the PIA forwarded port (so peers can connect)
1. **Enable** `VPN_PORT_FORWARDING=on` in gluetun (as in the compose above). Gluetun will request a forwarded port from PIA if the selected gateway supports it. citeturn1view0  
2. Gluetun stores the assigned forwarded port in the container (historically in `/tmp/gluetun/forwarded_port` or via the built-in API endpoint). You can query gluetun’s API endpoint (e.g. `http://<host>:5555/v1/openvpn/portforwarded`) or read the forwarded port file from the gluetun volume. Community notes show using this file/API and copying the port into qBittorrent’s PEERPORT setting. citeturn2search0turn2search2  
3. **Automate** this: create a tiny script or container that waits for gluetun to expose the forwarded port, then issues an API call to qBittorrent (or edits its config) to set `PEERPORT` and restart the client if necessary. Several community guides and forum posts demonstrate this pattern; it’s common because PIA rotates forwarded ports periodically. citeturn2search1turn2search7

Example of the quick manual check (inside the gluetun container):
```bash
# run a shell into gluetun
docker exec -it gluetun /bin/sh
# check forwarded port file (if present)
cat /tmp/gluetun/forwarded_port
# or use the API if you've exposed it
curl http://localhost:5555/v1/openvpn/portforwarded
```

---

# Kill-switch, DNS leak protection and safety
- Gluetun includes a built-in firewall/kill-switch and DNS over TLS features; configure `FIREWALL_*` env vars or the gluetun config to allow only VPN+LAN traffic and to block leaks. This prevents your downloader from accidentally using the host network if the VPN drops. citeturn1view0  
- Always test for leaks (DNS/web) after bringing the stack up using containers that call `https://ifconfig.co` or `https://ipleak.net` from the downloader container namespace to confirm WAN IP is the VPN IP.

---

# Usenet (NZB) specifics
- Usenet clients (SABnzbd, NZBGet) can also be routed through gluetun. Usenet generally does not require port forwarding, but you should still route them through PIA if your goal is to protect the host IP for those downloads. Use the same `network_mode: "service:gluetun"` pattern for SABnzbd/NZBGet. (Usenet downloads are normally direct TCP connections to the news server, not peer-to-peer.)

---

# Practical troubleshooting pointers & references
- If your torrent client shows `port: 0` or `no port`, first confirm gluetun acquired a forwarded port (check `/tmp/gluetun/forwarded_port` or gluetun API). If it didn’t, switch PIA region or protocol (OpenVPN) and reattempt. citeturn2search0turn0search14  
- If you want the more modern WireGuard performance, test it — but expect extra steps or community workarounds to get port-forward reliably with PIA + gluetun. There are several open issues and community threads about that. citeturn0search10turn2search3

---

# Small automation script idea (auto-set PEERPORT for qBittorrent)
This is the usual community pattern in words (not a full script — adapt for your environment):

1. Loop: wait until `./gluetun/config/forwarded_port` (or `docker exec gluetun curl .../portforwarded`) returns a non-zero port.  
2. Call qBittorrent WebAPI (`/api/v2/transfer/setPreferences` or edit config) to set `session\_port` (a.k.a. `PEERPORT`) to that value.  
3. Save and optionally restart qBittorrent if needed.  
4. Run the script at container start, or run it as a tiny helper container that depends_on gluetun.

Community guides and forum posts show ready examples for qBittorrent + gluetun; use them as a template. citeturn2search7turn2search13

---

# Final checklist before you go live
- [ ] Verify PIA account credentials and choose a region that advertises port-forwarding. citeturn0search14  
- [ ] Deploy gluetun with `VPN_PORT_FORWARDING=on` and test that a forwarded port is obtained. citeturn1view0  
- [ ] Attach your downloader containers to gluetun (`network_mode: "service:gluetun"`) so all traffic is via the VPN. citeturn1view0  
- [ ] Implement the port update automation to keep qBittorrent’s peer port synced with gluetun’s forwarded port. citeturn2search0  
- [ ] Confirm kill-switch + DNS leak protection are working. citeturn1view0  
- [ ] Only download content you have the legal right to obtain.

---

If you’d like, I can:
- produce a ready-to-run `docker-compose.yml` tailored to your exact paths, PIA username (placeholder), and whether you want OpenVPN or WireGuard; **or**
- draft the small helper script to auto-read `forwarded_port` and push it into qBittorrent’s WebAPI (I’ll provide a tested shell or Python example).  

Which would you prefer (OpenVPN or WireGuard in gluetun), and do you want the automation script for updating qBittorrent’s peer port?

### User (2025-10-02 18:01:39)

Are there any automated git repos that just do all of this stuff for you? including the sonar, radar, usenet, and pia stuff (for legal reasons lets say)

### Assistant (2025-10-02 18:01:40)

{"search_query": [{"q":"docker compose plex sonarr radarr qbittorrent gluetun privateinternetaccess github"}, {"q":"media server docker-compose sonarr radarr nzbget gluetun github"}, {"q":"haugene transmission-openvpn github"}, {"q":"autobrr docker compose sonarr radarr gluetun docker media stack github"}, {"q":"docker media server all-in-one repo plex sonarr radarr radarr nzbget qbittorrent gluetun 'docker-compose' 'github' 'plex'"}], "response_length":"short"}

### Assistant (2025-10-02 18:01:48)

{"search_query": [{"q":"qmcgaw gluetun github"}, {"q":"AdrienPoupa docker-compose NAS PIA Traefik github AdrienPoupa"}], "response_length":"short"}

### Assistant (2025-10-02 18:01:54)

Short answer: **Yes** — there are several community-maintained GitHub repositories / docker-compose “stacks” that bundle Plex/Jellyfin + the *arr* apps (Sonarr, Radarr, Prowlarr, Jackett) + downloaders (qBittorrent / SABnzbd / NZBGet) and a VPN container (PIA via Gluetun or older OpenVPN-based images). Below I list the best-known, actively used examples, notes on what they do, pros/cons, and exactly what to check before you run one. (I’ll also offer a next step if you want me to produce a ready-to-run compose tailored to your `/srv` paths + PIA creds.)

# Repos / stacks you should look at
(Quick summary first, then short pros/cons and what to check.)

1. **AdrienPoupa / docker-compose-nas** — “one file” Docker Compose NAS: Sonarr, Radarr, Prowlarr, qBittorrent, Jellyfin (or Plex forks), Traefik for HTTPS, and **PIA** WireGuard/OpenVPN integration in the examples. This is one of the most popular all-in-one references and explicitly includes PIA configuration examples. citeturn1search1turn1search16  
   *Pros:* well-documented, lots of community forks, meant to be copy-and-edit.  
   *Watch:* confirm the PIA server/port-forwarding behavior & pick an appropriate region; read the repo issues for recent compatibility notes. citeturn1search16

2. **qdm12 / gluetun** — not a whole stack but the de-facto modern VPN client container that many stacks use (supports Private Internet Access, many providers, OpenVPN/WireGuard, DNS over TLS, built-in port-forward support). Use Gluetun as the VPN core and attach downloaders to it. (This repo is the upstream image used by many compose stacks.) citeturn1search0turn1search2  
   *Pros:* modern, actively maintained, good docs, supports provider-specific options.  
   *Watch:* choose OpenVPN for stable PIA port-forwarding unless you validate WireGuard behavior for your provider/region.

3. **haugene / docker-transmission-openvpn** — older but battle-tested OpenVPN+Transmission image (very widely used historically). Good if you want the traditional combined container approach for Transmission+VPN. Many Kubernetes/compose examples fork this pattern. citeturn0search2turn0search6  
   *Pros:* easy, well-known.  
   *Watch:* some users report issues with newer releases; check issues/versions.

4. **DonMcD / ultimate-plex-stack** — a curated “ultimate” plex + servarr stack repo that stitches together Plex, Sonarr, Radarr, Prowlarr, and optional VPN bits. Good template for production-ready layout. citeturn0search3

5. **johnwyles / docker-compose-media-center** and **vaeyo / MediaServer-DockerComposeFiles** — collections of compose examples that aim to stand up the entire Servarr ecosystem (Sonarr/Radarr/Jackett/Prowlarr/Downloaders) and often include a VPN container option. Useful as smaller bite-sized templates. citeturn0search1turn0search19

6. **Morzomb / All-jellyfin-media-server** and **geekau / MediaStack** — complete compose stacks that explicitly include Gluetun (VPN) + Sonarr/Radarr + qBittorrent + Jellyfin/Plex alternatives. Good if you want a more “Jellyfin-first” setup but they’re easily switched to Plex. citeturn0search18turn0search7

# What these repos usually handle automatically
- Creating the Docker Compose layout (services, volumes, env). citeturn1search1turn0search3  
- Wiring *arrs* (Sonarr/Radarr/Prowlarr) to downloaders (qBittorrent/NZBGet/SABnzbd) via hostnames in the Docker network. citeturn0search14  
- A VPN container (Gluetun or OpenVPN image) and examples of routing downloaders through it (either `network_mode: "service:gluetun"` or putting containers on the gluetun network). citeturn1search0turn0search4

# Caveats — what you MUST check before using any “automated” repo
1. **Legal & ethical:** only configure and use these tools for content you have the right to download. I can’t help you evade enforcement.  
2. **PIA port-forwarding:** if you depend on incoming P2P connectivity, pick a repo/config that uses Gluetun/OpenVPN + PIA and verify that the chosen PIA region supports port forwarding — not all do. (Gluetun has flags for port-forwarding.) citeturn1search0turn1search2  
3. **Maintenance & forks:** many “one-click” stacks are forks — check latest activity (commits, issues) before trusting them for production. Some popular images (e.g., haugene/transmission-openvpn) have had regressions in the past. citeturn0search12turn1search6  
4. **Network exposure:** make sure the compose config doesn’t accidentally expose your downloaders/Plex UI to the public internet. Many repos include Traefik/Caddy configs — review them and only enable public HTTPS if you want that. citeturn1search1  
5. **Secrets management:** these repos typically leave placeholders for VPN username/passwords and API keys — never commit those into a public fork. Use `.env` or Docker secrets.

# Recommendation (practical)
- If you want a mature, widely-used reference: start with **AdrienPoupa/docker-compose-nas** — it already includes PIA examples and *arr* apps and has lots of community forks and troubleshooting threads. citeturn1search1  
- Use **qdm12/gluetun** as your VPN container (replace older haugene/openvpn containers unless you have a specific reason to use haugene). Gluetun supports PIA and has modern features (DNS over TLS, WireGuard/OpenVPN, port forwarding). citeturn1search0turn1search2

# Want me to do one of the following now?
Pick one and I’ll produce it right away (ready for you to paste/run on your Ubuntu box):

- Option A — **Trimmed AdrienPoupa compose**: I take AdrienPoupa’s compose and produce a sanitized, Plex-focused `docker-compose.yml` with Gluetun (PIA via OpenVPN), Sonarr, Radarr, qBittorrent, Prowlarr, Jackett, and placeholders for your PIA creds + paths — all tuned for `/srv` and non-public access via Tailscale/WireGuard or private LAN. (I’ll ensure `network_mode: "service:gluetun"` for downloaders and add a small helper script stub to read gluetun’s forwarded port.) citeturn1search1turn1search0

- Option B — **Gluetun + Sonarr/Radarr + NZBGet/qBittorrent minimal compose**: minimal, well-commented compose focusing on the VPN routing pattern, port forwarding automation script for qBittorrent, and safety checks for DNS leaks (good for learning/testing). citeturn1search0turn0search4

- Option C — I can **walk you through** validating a chosen GitHub repo’s health (open issues, last commit, forks) and customizing it securely for Ubuntu 24.04 (file paths, systemd service, UFW rules).

Which option do you want? If you pick A or B, tell me:
- do you prefer **OpenVPN** or **WireGuard** for PIA (I’ll default to OpenVPN for port-forward reliability), and  
- confirm your preferred media paths (I’ll assume `/srv/media` and `/srv/docker` if you don’t say otherwise).

### User (2025-10-02 18:04:21)

do any of those bundle packages include a service like ombi (maybe a 2025 version?) that lets outside clients use an app to request things that eventually are queued in sonar and radar and brought into your plex network?

### Assistant (2025-10-02 18:04:21)

search("do any of those bundle packages include a service like ombi (maybe a 2025 version?) that lets outside clients use an app to request things that eventually are queued in sonar and radar and brought into your plex network?")

### Assistant (2025-10-02 18:04:23)

Yes — many of the “all-in-one” media stacks *do* include or support an “request management” service (Ombi or its alternatives) so outside users can request shows/movies, which then feed into your *arr (Sonarr / Radarr) workflow. It’s quite common in the community.  

Here’s what I found and how it generally works, along with caveats and advice (especially as of 2025).

---

## What is Ombi (or equivalent) and how it integrates

- **Ombi** is a request-management / user request portal: it lets users (non-admins) log in, see your Plex library, request new titles/episodes/seasons, view status, etc. It then triggers actions (via APIs) into Radarr/Sonarr to add/monitor the requested content. citeturn0search1turn0search3  
- The LinuxServer Ombi image is actively maintained; their releases page shows v4.47.x as of recent updates. citeturn0search9  
- The Ombi docs include examples of how it would run in Docker, talk to a database, and connect to your media server environment. citeturn0search3  

So yes — Ombi is still alive and suitable for 2025.  

---

## Examples of stacks that include Ombi (or support it) in their “bundle”

- I found a **docker-compose gist** that includes Plex, Transmission, Sonarr, Radarr, Jackett, *and* Ombi in a single compose file. That’s a proof that people already build that as a “monolithic” stack. citeturn0search10  
- The **AdrienPoupa/docker-compose-nas** project is a strong candidate — their README & compose layout list many services: Sonarr, Radarr, Prowlarr, qBittorrent, Jellyfin, PIA VPN, Traefik, etc. In their “Applications” section, they mention “optional services”: under “Optional Services” they list **SABnzbd**, etc. They *don’t explicitly* list Ombi in their main list in the top summary, but their stack is flexible and they do mention “homepage / dashboard” and request apps (e.g. “Jellyseer” which is a request-frontend for Jellyfin) in optional modules. citeturn0search2  
- Another integration is in “AutoPirate” stacks, where they explicitly include Ombi as one of the components. E.g., in the AutoPirate (swarm) stack, there's a snippet showing how to add Ombi to the compose, tie it into internal networks, and expose it via Traefik. citeturn0search14  

Thus, yes — you can find or adapt a stack that includes Ombi (or a variant) along with Plex, Sonarr, Radarr, VPN, etc.

---

## What to watch out for when using a bundled request system

When using a “bundle + Ombi (or equivalent)” setup, here are key challenges or configuration points you’ll want to validate or customize:

| Concern | Notes / what to verify |
|---|---|
| **API connectivity** | Ombi must talk to Sonarr, Radarr, Plex (for library status), possibly Prowlarr/Jackett. Make sure network aliases or service names match (in Docker) so `ombi` → `radarr:7878` etc. |
| **User permissions** | You’ll need to configure permissions so when a user requests, it can be added to Sonarr/Radarr under your desired quality/profile settings. |
| **Database / persistence** | Ombi often uses an internal database or external (e.g. MySQL). Ensure you map `config` volumes to persist. |
| **Upgrades & compatibility** | When apps evolve, API changes can break request flows — ensure the stack you choose is actively maintained. The issue tracker for Ombi had a bug where after an update it couldn't connect to MySQL due to SSL handshake issues. citeturn0search13 |
| **Security & exposure** | Don’t expose the request system publicly without authentication. Use reverse proxies, TLS, and/or VPN or network isolation. Make sure the request portal doesn’t provide admin access to your backend. |
| **VPN routing / container networks** | If you’re routing downloaders through a VPN container, make sure Ombi can still talk to the `arr` apps (over internal network) even if those apps are “behind” VPN or a separate network segment. |
| **Conflict with other request systems** | Some stacks use **Overseerr** (another popular request system) or **Jellyseerr** (Jellyfin variant) instead of Ombi. If the bundle uses one of those, you might prefer switching to it. |

---

## My take: recommending an approach in 2025

Given what you want (Plex + request system + arr + VPN), here’s what I’d do if I were you:

1. **Pick a modern base stack** that already supports modular inclusion of “optional services” (like AdrienPoupa’s).  
2. **Enable/insert Ombi (or Overseerr / Jellyseerr)** into the stack via the optional module, link the networks so it can talk to Sonarr/Radarr, and expose its UI only behind your secure frontend (Caddy/Traefik) or your private mesh (WireGuard/Tailscale).  
3. **Test the request → Sonarr/Radarr path** manually and observe logs to confirm correct API calls.  
4. **Keep the stack upgradable** (watch the request system’s issues and update constraints) so you don’t break the request path on upgrades.

---

If you like, I can **generate a fully integrated `docker-compose.yml`** for your Ubuntu server with:

- Plex, Sonarr, Radarr, Prowlarr, Jackett  
- Gluetun (PIA) for routing downloads  
- Ombi (latest version) as the request portal  
- Reverse proxy (Caddy or Traefik) for exposing Plex / Ombi securely  

And I’ll annotate how the request flow works (user → Ombi → Radarr/Sonarr → downloaders → Plex). Do you want me to produce that ready-to-run file for you?

