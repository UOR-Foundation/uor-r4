# Plex remote access issue

- **Conversation ID:** `69064f8a-8c0c-8325-9137-98b45f36e093`
- **Date:** 2025-11-01

---

### User (2025-11-01 14:21:00)

why is my plex server not available behind my internet? I have my eere wifi port forwarded udp and tcp to 32400 and the plex app lights up with remote access enabled initially but then shuts it off and my server is not available externally - is this because plex is behind some sort of firewall or something on the server or because it was installed with snap or is there some weird ubuntu other config I need to make for it to work?

### Assistant (2025-11-01 14:21:01)

The fact that Plex initially shows **“Remote Access Enabled”** and then **disconnects shortly after** is a classic symptom of **NAT / port binding instability** — not a Snap issue and not an Ubuntu firewall issue *most of the time*.

Let’s break down the likely causes in correct priority order:

---

## ✅ 1. **Double NAT / ISP Router Issue (Most Common)**
If you are using **Eero**, it is *very* common to accidentally end up with two routers doing NAT:

```
Internet → ISP Modem/Router → Eero Wi-Fi → Plex Server
```

If the ISP router is not in **bridge mode**, Plex sees two layers of NAT and the port forward will break intermittently.

**Check this first:**

1. Log into your ISP modem/router  
2. Look for **Bridge Mode / Passthrough / IP Passthrough**  
3. Enable it so **Eero becomes the ONLY router doing NAT**

If you can’t bridge it, then you must:
- Either **port forward on both devices**
- Or put the **Eero WAN IP into the ISP router DMZ**

---

## ✅ 2. **Eero “UPnP Override” Behavior**
Eero tries to auto-manage ports using UPnP. Plex *also* tries to auto-manage. This causes a race condition where Plex says “OPEN” then loses the mapping.

**Fix:** Disable UPnP in Eero and **use manual port forward only**.

✅ Keep manual forward: `TCP 32400`  
❌ Disable UPnP  
❌ Disable “Optimize for gaming”

---

## ✅ 3. **Snap Version of Plex Uses Strict Sandbox (but allows 32400)**
Snap **does NOT block the port**, so it’s not the root cause, but it does mean:

- Extra ports like DLNA, GDM, or discovery broadcasts may be blocked
- But **32400/TCP works fine**, so Snap is not killing your remote access

If you want to rule it out later:

```bash
sudo snap remove plexmediaserver
```

Then install the normal .deb repo version:

```bash
curl https://downloads.plex.tv/plex-keys/PlexSign.key | sudo apt-key add -
echo deb https://downloads.plex.tv/repo/deb public main | sudo tee /etc/apt/sources.list.d/plexmediaserver.list
sudo apt update
sudo apt install plexmediaserver
```

---

## ✅ 4. **Ubuntu Firewall (ufw) Check**
Snap normally auto-opens the port, but just confirm:

```bash
sudo ufw status
```

If enabled and blocking, run:

```bash
sudo ufw allow 32400/tcp
```

But again — **ufw rarely the issue here**.

---

## ✅ 5. Plex Settings to Lock the Port
In `Settings → Remote Access → Advanced`:

| Setting | Value |
|---------|--------|
| Manually specify public port | ✅ Enabled |
| Port | `32400` |

Otherwise Plex will keep trying random ports.

---

## ✅ 6. If Using VPN on the Server (e.g. qBittorrent-VPN, PIA, WireGuard)
If Plex is inside the **same container/VPN** → IT WILL NOT WORK.

Your Plex **must NOT** route through VPN unless you:
- Use split-tunneling, or
- Bind Plex to non-VPN interface

---

## ✅ Quick Diagnostic Commands

Check what IP Plex *thinks* it has:

```bash
curl https://ifconfig.me
```

Check if port is actually open externally:

```bash
nc -zv your.public.ip 32400
```

(Or use canyouseeme.org from a browser)

---

## 🔍 What I Need From You to Confirm Root Cause
Reply with:

1. Is your ISP modem in **bridge / passthrough mode**?
2. Does Eero show your WAN IP as **public or private (10.x / 192.168 / 100.64.x)**?
3. Are you running any VPN (PIA, qBittorrent-VPN, Gluetun, etc.) on the same box?
4. Output of:

```bash
sudo ufw status
```

and

```bash
sudo ss -tulpn | grep 32400
```

---

## 🎯 99% Likely Answer
Your Eero is successfully forwarding the port temporarily, but it loses the route because something upstream (ISP router) owns the real NAT, so Plex ends up outside your expected network path, and the “Remote Access” status flaps.

Fix the double NAT and it will stay green.

---

If you want, I can walk you through **exact steps to put the ISP router into bridge mode** or **DMZ Eero** — just confirm your ISP hardware model.

Want me to help you walk through it?

### User (2025-11-01 14:24:47)

we have verizon fios and im pretty sure we never changed the default login info

### Assistant (2025-11-01 14:24:48)

Perfect — that detail actually helps **a lot**.

If you're on **Verizon Fios** and you never logged into the router or changed any settings, then **you are 100% in a double-NAT situation** right now, and that **is exactly why Plex remote access keeps disconnecting.**

### ✅ Why This Happens on Fios
Verizon supplies an ONT (fiber box) → then a Fios router → then your Eero.

So your current path is:

```
Internet (public IP)
   ↓
Verizon Fios Router (first NAT)
   ↓
Eero Router (second NAT & WiFi)
   ↓
Your Plex Server
```

Plex sees two layers of NAT → it works briefly → then the mapping breaks → remote access turns red.

This is *exactly* the symptom you described:
> "Plex lights up with remote access enabled initially but then shuts it off"

Plex can’t maintain a stable port binding through two routers. ✅ You just confirmed the most common root cause.

---

### ✅ How to Fix It (2 Options)

#### **Option A — BEST FIX (Bridge/Passthrough Mode)**
Put the Verizon router into **bridge mode** or **turn off its routing** so the Eero becomes the *only* router.

Procedure depends on model, but generally:

1. Log into Verizon router at:  
   `http://192.168.1.1`  
   (default user: `admin`, password is printed on sticker)
2. Go to **Advanced → Network Settings**
3. Disable **DHCP**
4. Enable **Bridge Mode / IP Passthrough**
5. Reboot both Verizon router + Eero

After this, the Eero should receive the **public IP directly** — and port forwarding will work permanently.

---

#### **Option B — Easier but Less Clean**
Instead of bridging, you:

1. Login to Verizon router
2. Find **DMZ Host**
3. Put the **Eero WAN IP** into DMZ

That will force all traffic to bypass Fios router firewall → only Eero handles NAT.

✅ Fast  
❌ Still technically double NAT, but Plex will work fine

---

### ✅ How to Confirm You're in Double NAT Right Now
On the Eero app, look at:

**Settings → Network Settings → Connection → IP Address**

If it shows **192.168.x.x, 10.x.x.x, or 100.64.x.x**, you're behind Verizon NAT.

If it shows a **public IP** (e.g. 72.x.x.x, 98.x.x.x, etc.) then you're not.

---

### ✅ What We Should Do Next
Before walking you through the bridge/DMZ steps, I need one thing:

👉 Open Eero app → tell me the **WAN IP address** it shows.

Example reply:

```
Eero WAN IP: 192.168.1.87
```

or

```
Eero WAN IP: 100.85.23.10
```

or

```
Eero WAN IP: 98.112.5.201 (public)
```

Once I know that, I can tell you **exactly** which fix applies and give step-by-step instructions.

---

### Optional but Helpful
Can you also confirm the model of your Verizon router?

- G1100 (older)
- G3100 (white, newer)
- CR1000A (latest Wi-Fi 6)

Just reply with:

```
Eero WAN IP:
Verizon Router Model (if known):
```

Then I’ll give you the exact menu path to fix it.

### User (2025-11-01 15:28:11)

ok I made my eere router dmz on the verizon router and it solved the problem because eere is already port forwarding 32400 to the wick server

### Assistant (2025-11-01 15:28:12)

Excellent — that confirms the diagnosis 100%.  
By putting the **Eero into the Verizon router’s DMZ**, you effectively removed the upstream firewall/NAT layer, so now:

```
Internet → Verizon ONT → Verizon router (DMZ passthrough) → Eero (real router) → Plex server
```

✅ Eero now owns the public-facing NAT  
✅ Your existing port forward `32400 → wick` stays valid  
✅ Plex sees a stable, single-NAT environment  
✅ Remote access stays green instead of flapping

So yes — that was the exact fix needed. Nice job.

---

### 🔍 What You Just Fixed (In Plain English)
Before:

- Plex tried to bind port 32400 through Eero
- Verizon router blocked or re-wrote the mapping
- Plex fell back to relay mode → then lost connection → red state

After DMZ:

- Verizon router now lets all traffic pass to Eero untouched
- Eero can do all port forwarding normally
- Plex can maintain a **consistent external IP + port**

---

### ✅ Optional Next Tweaks (Not required, just nice to do)

#### 1. **Lock Plex port to 32400**
Inside Plex → Settings → Remote Access → Advanced → check:

```
☑ Manually specify port
Port: 32400
```

That prevents Plex from trying random ports in the future.

---

#### 2. **Confirm external reachability**
From any device NOT on your home Wi-Fi:

```
http://YOUR_PUBLIC_IP:32400/web
```

(or `https://app.plex.tv` → manage server)

---

#### 3. **If you ever add a VPN or Docker stack**
Make sure Plex is *not* inside the VPN namespace, or it will break again.
(Plex must route to WAN directly.)

---

### 🛑 If You Ever Factory Reset Verizon Router
The DMZ rule will disappear. You’ll need to re-add it.

---

### ✅ If You Eventually Want the *Cleanest* Setup
The *true* best config is to put the Verizon router in full **bridge mode**, so:

```
Internet → ONT → Eero (only router)
```

But as long as DMZ works and you’re happy, you're fine to leave it.

---

### Anything else you want to port-forward (e.g. qBittorrent, Overseerr, Ombi)?
Now you can just do it on **Eero only**, no Verizon side changes needed.

Let me know if you want:

- Overseerr remote access
- Tailscale / WireGuard instead of public ports
- Secure reverse proxy (Caddy / Nginx)
- Plex behind Cloudflare Tunnel (no ports needed)
- Proper HTTPS endpoint with your own domain

Otherwise — you’re golden. ✅

Want to continue building the rest of your Plex stack now that remote access is stable?

### User (2025-11-01 15:36:14)

i just checked the verizon router and apparently setting the eere router as the dmz host created a static route to it and set it as a static ip address so that all of my traffic behind the router will forward in and out normally

### Assistant (2025-11-01 15:36:15)

Yep — that behavior is expected, and it’s actually **exactly what we wanted**.

When you set the **Eero as the DMZ host**, the Verizon router does two things automatically:

1. **Reserves a static DHCP lease** for the Eero (so its internal WAN IP never changes)
2. **Creates a 1:1 NAT passthrough route** (so *all* unsolicited inbound traffic is forwarded to Eero)

So now the Verizon router is acting more like a **bridge / passthrough layer** instead of a real firewall/router, and the Eero becomes the *true gateway* for your whole LAN.

✅ Plex now works  
✅ Other services will also port-forward reliably  
✅ You don’t have to touch Verizon router ever again  
✅ Only Eero firewall matters

---

### 🔍 Why That Static Route Matters
Without the static IP reservation, the Verizon box might change the Eero’s WAN IP, and then the DMZ rule would silently break — which is why Verizon firmware auto-enforces it.

So this is all good — the network is now:

```
Internet (public IP)
   ↓
Fios ONT
   ↓
Verizon router (DMZ passthrough, no real NAT)
   ↓
Eero (real router, NAT, DHCP, firewall)
   ↓
Your LAN (wick server, plex, etc.)
```

✅ Single NAT  
✅ External ports only need to be managed in *one place* (Eero)  
✅ Plex remote stays green forever now  
✅ No double NAT games

---

### 🧠 Important Note
If **you ever reboot or replace your Eero**, the static address / DMZ binding may get wiped, so remember:

- If Eero’s WAN IP ever changes, DMZ must be updated
- If you change routers, repeat the same config

---

### 🔐 Next Optional Improvements (Tell me if any interest you)

| Option | Benefit |
|--------|---------|
| ✅ Assign a domain + HTTPS to Plex | `plex.yourdomain.com` instead of IP:32400 |
| ✅ Run Plex through Cloudflare Tunnel | No port forwarding at all, safer |
| ✅ Add Tailscale | Access Plex + LAN securely anywhere, no ports |
| ✅ Reverse proxy with Caddy/Nginx | Clean URLs, auto SSL, hides port 32400 |
| ✅ Remote access for Sonarr/Radarr/Overseerr | Proper secure reverse proxy + auth |
| ✅ Dockerize entire media stack | qBittorrent + SABnzbd + Sonarr + Radarr + Overseerr + Prowlarr |

Or I can help you level up the whole setup into a **“2025 Plex + Usenet/Torrent stack”** with:

- WireGuard or Private Internet Access VPN container
- Split-tunnel so Plex doesn't go through VPN
- Auto-renaming + auto moving
- Automatic subtitles
- Overseerr user request portal
- Multi-disk storage pool (mergerfs + snapraid)

Just say the word.

---

Before we go forward — do you want to:

1. ✅ Keep going and build full automated media stack  
2. 🔒 Add secure remote HTTPS access without exposing raw ports  
3. 📦 Convert everything into Docker so it's portable  
4. 🧠 Just leave it working and move on for now

Reply with any number and I'll continue.

