# Disable sleep Ubuntu 24.04

- **Conversation ID:** `68e28f2a-bda4-8331-abfe-d26902df777f`
- **Date:** 2025-10-05

---

### User (2025-10-05 11:31:17)

how to make ubuntu 24.04 never sleep or hibernate or whatever so the ssh is always available and always on via ssh terminal

### Assistant (2025-10-05 11:31:18)

To make sure your **Ubuntu 24.04** system never sleeps or hibernates (so SSH is always available), you’ll want to disable all power management features that could suspend the machine. Here are the key steps:

---

### 1. Disable sleep/hibernate systemd targets
Run these commands to mask sleep, suspend, and hibernate across the system:

```bash
sudo systemctl mask sleep.target suspend.target hibernate.target hybrid-sleep.target
```

This ensures no service can trigger those states.

---

### 2. Disable automatic sleep via `systemd-logind`
Edit the `logind.conf` file:

```bash
sudo nano /etc/systemd/logind.conf
```

Set or update these lines:

```
HandleSuspendKey=ignore
HandleLidSwitch=ignore
HandleLidSwitchDocked=ignore
IdleAction=ignore
```

Save and restart the service:

```bash
sudo systemctl restart systemd-logind
```

---

### 3. Disable GNOME/desktop power saving (if GUI is installed)
For desktops, Ubuntu uses `gsettings` or `dconf`. Disable screen blank and sleep:

```bash
gsettings set org.gnome.settings-daemon.plugins.power sleep-inactive-ac-type 'nothing'
gsettings set org.gnome.settings-daemon.plugins.power sleep-inactive-battery-type 'nothing'
gsettings set org.gnome.desktop.session idle-delay 0
```

---

### 4. Prevent kernel from auto-suspending devices
Add this to your GRUB config to disable USB and PCI autosuspend (sometimes laptops still sleep otherwise):

Edit:

```bash
sudo nano /etc/default/grub
```

Update this line:

```
GRUB_CMDLINE_LINUX_DEFAULT="quiet splash nohibernate"
```

Then update grub:

```bash
sudo update-grub
```

---

### 5. Confirm the system will never sleep
Run:

```bash
systemctl status sleep.target suspend.target hibernate.target hybrid-sleep.target
```

They should all show as **masked**.

---

✅ With these steps, your Ubuntu 24.04 server will **never suspend, hibernate, or auto-sleep**, keeping SSH permanently available.  

Would you like me to also give you a **one-liner script** that applies all of these at once so you don’t have to do them step by step?

### User (2025-10-05 11:32:18)

yes

### Assistant (2025-10-05 11:32:18)

Perfect 👍 here’s a **one-liner script** you can copy/paste into your Ubuntu 24.04 terminal to disable *all* forms of sleep/hibernate/idle suspend in one go:

```bash
sudo systemctl mask sleep.target suspend.target hibernate.target hybrid-sleep.target && \
sudo sed -i 's/^#\?HandleSuspendKey=.*/HandleSuspendKey=ignore/' /etc/systemd/logind.conf && \
sudo sed -i 's/^#\?HandleLidSwitch=.*/HandleLidSwitch=ignore/' /etc/systemd/logind.conf && \
sudo sed -i 's/^#\?HandleLidSwitchDocked=.*/HandleLidSwitchDocked=ignore/' /etc/systemd/logind.conf && \
sudo sed -i 's/^#\?IdleAction=.*/IdleAction=ignore/' /etc/systemd/logind.conf && \
sudo systemctl restart systemd-logind && \
gsettings set org.gnome.settings-daemon.plugins.power sleep-inactive-ac-type 'nothing' && \
gsettings set org.gnome.settings-daemon.plugins.power sleep-inactive-battery-type 'nothing' && \
gsettings set org.gnome.desktop.session idle-delay 0 && \
sudo sed -i 's/^GRUB_CMDLINE_LINUX_DEFAULT=.*/GRUB_CMDLINE_LINUX_DEFAULT="quiet splash nohibernate"/' /etc/default/grub && \
sudo update-grub
```

---

### What it does:
- Masks all `systemd` sleep targets.  
- Updates `/etc/systemd/logind.conf` to ignore lid/suspend keys and idle actions.  
- Restarts `systemd-logind` so changes take effect immediately.  
- For GUI installs: disables GNOME power management sleep + idle delay.  
- Adds `nohibernate` to GRUB boot params, then updates GRUB.  

---

After running this, reboot once:

```bash
sudo reboot
```

And your machine should **never sleep or hibernate**, keeping SSH always alive.  

Do you want me to also make you a **reversible script** (to restore sleep/hibernate) in case you ever change your mind later?

### User (2025-10-05 11:35:50)

this was the result of your script:

Created symlink /etc/systemd/system/sleep.target → /dev/null.
Created symlink /etc/systemd/system/suspend.target → /dev/null.
Created symlink /etc/systemd/system/hibernate.target → /dev/null.
Created symlink /etc/systemd/system/hybrid-sleep.target → /dev/null.

(process:4130): dconf-WARNING **: 11:32:59.934: failed to commit changes to dconf: Cannot autolaunch D-Bus without X11 $DISPLAY

(process:4136): dconf-WARNING **: 11:32:59.938: failed to commit changes to dconf: Cannot autolaunch D-Bus without X11 $DISPLAY
Sourcing file `/etc/default/grub'
Generating grub configuration file ...
Found linux image: /boot/vmlinuz-6.14.0-33-generic
Found initrd image: /boot/initrd.img-6.14.0-33-generic
Found linux image: /boot/vmlinuz-6.14.0-32-generic
Found initrd image: /boot/initrd.img-6.14.0-32-generic
Found memtest86+ 64bit EFI image: /boot/memtest86+x64.efi
Warning: os-prober will not be executed to detect other bootable partitions.
Systems on them will not be added to the GRUB boot configuration.
Check GRUB_DISABLE_OS_PROBER documentation entry.
Adding boot menu entry for UEFI Firmware Settings ...
done

### Assistant (2025-10-05 11:35:53)

Thanks for the output—that helps.

Those two `dconf` warnings are harmless on a headless/server install. They happened because the `gsettings` lines in the one-liner require a desktop session + DBus/X11. Since you’re (likely) on a server/SSH session, you can ignore them. Everything that actually matters for preventing sleep/hibernate **did succeed**:

- All suspend/hibernate targets are **masked** (the symlinks to `/dev/null` confirm it).
- `logind.conf` was updated and `systemd-logind` restarted.
- GRUB was updated with `nohibernate`.

If you want to verify and harden a bit more, do this:

### 1) Verify nothing can sleep/hibernate
```bash
systemctl status sleep.target suspend.target hibernate.target hybrid-sleep.target
loginctl show-user "$USER" -p IdleHint -p IdleSinceHint
systemd-inhibit --list
```
- The four targets should be **masked**.
- `IdleHint` is informational only.
- `systemd-inhibit --list` should show no suspend inhibitors needed (because suspend is masked).

### 2) (Server-only) Remove the GUI bits we don’t need
Since you don’t use GNOME here, just skip the `gsettings` lines entirely. Nothing else to do.

### 3) Keep SSH sessions resilient (so “always on” also feels “always connected”)
Add these to **/etc/ssh/sshd_config** (then restart SSH):
```bash
sudo sed -i 's/^#\?ClientAliveInterval .*/ClientAliveInterval 60/; s/^#\?ClientAliveCountMax .*/ClientAliveCountMax 5/; s/^#\?TCPKeepAlive .*/TCPKeepAlive yes/' /etc/ssh/sshd_config
sudo systemctl restart ssh
```
This sends a keepalive every 60s and tolerates brief network hiccups.

### 4) Optional: extra belt-and-suspenders (systemd sleep config)
Not strictly necessary (targets are already masked), but you can also say “no” at the sleep layer:
```bash
sudo bash -c 'cat >/etc/systemd/sleep.conf.d/never-sleep.conf <<EOF
[Sleep]
AllowSuspend=no
AllowHibernation=no
AllowHybridSleep=no
AllowSuspendThenHibernate=no
EOF'
sudo systemctl daemon-reload
```

### 5) Reversible “undo” script (if you ever change your mind)
```bash
sudo systemctl unmask sleep.target suspend.target hibernate.target hybrid-sleep.target
sudo sed -i 's/^HandleSuspendKey=.*/#HandleSuspendKey=suspend/' /etc/systemd/logind.conf
sudo sed -i 's/^HandleLidSwitch=.*/#HandleLidSwitch=suspend/' /etc/systemd/logind.conf
sudo sed -i 's/^HandleLidSwitchDocked=.*/#HandleLidSwitchDocked=ignore/' /etc/systemd/logind.conf
sudo sed -i 's/^IdleAction=.*/#IdleAction=suspend/' /etc/systemd/logind.conf
sudo rm -f /etc/systemd/sleep.conf.d/never-sleep.conf
sudo systemctl restart systemd-logind
sudo sed -i 's/GRUB_CMDLINE_LINUX_DEFAULT="quiet splash nohibernate"/GRUB_CMDLINE_LINUX_DEFAULT="quiet splash"/' /etc/default/grub
sudo update-grub
```

### TL;DR
You’re set. The warnings just mean “no desktop bus,” which is fine on servers. With the targets masked and logind updated, the box won’t suspend/hibernate—SSH will remain available. If you want, reboot once to test cleanly:
```bash
sudo reboot
```

### User (2025-10-05 11:39:26)

this was the output of your first check:

callard@wick:~$ systemctl status sleep.target suspend.target hibernate.target hybrid-sleep.target
loginctl show-user "$USER" -p IdleHint -p IdleSinceHint
systemd-inhibit --list
○ sleep.target
     Loaded: masked (Reason: Unit sleep.target is masked.)
     Active: inactive (dead)

○ suspend.target
     Loaded: masked (Reason: Unit suspend.target is masked.)
     Active: inactive (dead)

○ hibernate.target
     Loaded: masked (Reason: Unit hibernate.target is masked.)
     Active: inactive (dead)

○ hybrid-sleep.target
     Loaded: masked (Reason: Unit hybrid-sleep.target is masked.)
     Active: inactive (dead)
IdleHint=no
IdleSinceHint=1759678649000000
WHO                          UID USER PID  COMM            WHAT                                                     WHY                                                       MODE 
ModemManager                 0   root 1075 ModemManager    sleep                                                    ModemManager needs to reset devices                       delay
NetworkManager               0   root 982  NetworkManager  sleep                                                    NetworkManager needs to turn off networks                 delay
UPower                       0   root 1509 upowerd         sleep                                                    Pause device polling                                      delay
Unattended Upgrades Shutdown 0   root 1166 unattended-upgr shutdown                                                 Stop ongoing upgrades or perform upgrades before shutdown delay
gdm                          120 gdm  1570 gsd-media-keys  handle-power-key:handle-suspend-key:handle-hibernate-key GNOME handling keypresses                                 block
gdm                          120 gdm  1570 gsd-media-keys  sleep                                                    GNOME handling keypresses                                 delay
gdm                          120 gdm  1603 gsd-power       sleep                                                    GNOME needs to lock the screen                            delay

7 inhibitors listed.

### Assistant (2025-10-05 11:39:27)

That output looks good — you’ve got **all sleep/hibernate units masked** (they cannot be activated).  

The `systemd-inhibit --list` section shows why you still see “inhibitors”:  
- Things like **ModemManager, NetworkManager, UPower, GNOME daemons** register “delays” or “blocks” to coordinate with sleep if it were allowed.  
- But since `sleep.target`, `suspend.target`, `hibernate.target`, and `hybrid-sleep.target` are all **masked**, these inhibitors are irrelevant. They are saying “if sleep were possible, wait for me,” but since sleep is impossible, nothing happens.  

### ✅ What this means for you
- Your Ubuntu 24.04 system **will not sleep, suspend, or hibernate at all**.  
- SSH availability will persist as long as the machine is powered on and networked.  
- The “7 inhibitors listed” are just bookkeeping; they don’t override the masked targets.  

### 🔒 Optional hardening
If you want to *clean up* so even the inhibitors don’t register:  
1. Disable GUI daemons like `gdm`, `gnome-settings-daemon`, and `upower` if you’re on a pure server:  
   ```bash
   sudo systemctl disable --now gdm upower
   ```
   (Only do this if you don’t need a graphical login/power manager.)

2. Or leave them: they can’t force suspend anymore, so it’s safe.

---

⚡ TL;DR:  
Your system is already set. SSH will remain always on.  
The inhibitors you saw are **harmless noise** now that sleep/hibernate targets are masked.  

Do you want me to give you a **final cleanup script** that strips away `gdm/upower` inhibitors entirely (turning the box into a pure “server mode” that never even registers them)?

