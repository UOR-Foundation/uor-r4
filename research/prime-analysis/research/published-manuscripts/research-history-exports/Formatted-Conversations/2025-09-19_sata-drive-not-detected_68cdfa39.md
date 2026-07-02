# SATA drive not detected

- **Conversation ID:** `68cdfa39-f528-832a-ad46-cd184bbdc3ed`
- **Date:** 2025-09-19

---

### User (2025-09-19 20:58:10)

I have a samsung 1tb m.2 ssd plugged into the first M.2 slot (M.2_1) on my MSI Z170A motherboard. I also have a GTX 980 plugged into the first PCIe slot (I believe it's the PCIEx16 slot). I also have a normal SATA hard drive 10tb plugged into one of the SATA ports on the board. When I boot the computer, it recognizes all of the components in the BIOS except for the 10tb SATA drive. I can install Ubuntu 24 from a USB drive onto my 1tb M.2 drive, but neither the bios or the operating system recogonizes that there is a 10tb SATA drive connected as well. I have tried the SATA drive in every SATA port and have confirmed the power is connected correctly, but the drive will simply not show up - not even in the BIOS. I don't believe the cables, the powersupply, the harddrive, or any other components are bad so I am unsure why the 10tb SATA drive will not show up. I have tried both UEFI and Legacy modes with AHCI for the boot/drive configurations. I have tried every combination of the settings with the sata drive connected to different ports on the motherboard and it will not see the drive. I also have a CD drive connected to the same SATA bay, which is recognized by both the BIOS and the operating system (Ubuntu). I've tried plugging the SATA 10tb hard drive into the same slots as the CD drive and that did not work either. It's as if the BIOS sees the M.2 drive (in either M.2 slot) and immediately disregards any other SATA hard drive that is present. Do you have any ideas?

### Assistant (2025-09-19 20:58:10)

You’ve done a very thorough job ruling out power, cabling, and BIOS settings, so the fact that the 10 TB SATA disk is never detected in **any SATA port** (even ports where the CD/DVD works) points to a couple of quirks with your motherboard and drive combination. Here are the most likely culprits and steps to test:

---

### 1. Lane Sharing Between M.2_1 and SATA Ports
On the **MSI Z170A series**, the first M.2 slot (M.2_1) shares PCIe/SATA lanes with certain SATA ports.  
- When you populate M.2_1 with an NVMe SSD, the chipset **disables SATA ports 5 and 6**.  
- This means if your 10 TB HDD is plugged into SATA5/6, it will *never* show up.  

👉 Try plugging the HDD into **SATA1–SATA4** specifically and see if it’s detected.

---

### 2. Drive Capacity and UEFI
Older chipsets/BIOS versions sometimes don’t enumerate very large (>8 TB) drives properly.  
- Check if your MSI BIOS is fully updated (MSI released multiple Z170A BIOS revisions to improve large-drive compatibility).  
- If you’re running Legacy/CSM mode, try **UEFI only** with AHCI enabled.

---

### 3. Disk Formatting vs. BIOS Detection
The fact that it’s not even seen in BIOS suggests it’s not just a GPT/MBR partition issue. But if you eventually get it visible:  
- Ubuntu will only see the drive if it’s GPT-initialized. A brand new 10 TB might be uninitialized, but that alone wouldn’t explain BIOS invisibility.

---

### 4. Testing the Drive Itself
Even if you’re confident the drive isn’t bad:  
- Test the 10 TB HDD in another machine or with a USB-to-SATA adapter/dock.  
- If it spins up and is recognized elsewhere, you can rule out a defective drive.  
- If not, it might have firmware issues or need an RMA.

---

### 5. Optical Drive vs. HDD on Same Port
You mentioned swapping the HDD into the CD drive’s port with no luck. If the CD works on SATA2, and the HDD does not on SATA2, that strengthens the case for either:
- BIOS not supporting the large drive size without an update, or  
- the drive itself being the problem.

---

✅ **Next steps I’d suggest**:
1. Plug the 10 TB into **SATA1–SATA4 only** (not 5/6).  
2. Update the motherboard BIOS to the latest version from MSI’s support page.  
3. Boot BIOS in **pure UEFI** mode with AHCI enabled.  
4. If still not detected, test the HDD externally (USB dock or another PC).  

---

Would you like me to pull the exact **Z170A lane-sharing diagram** for your specific model (e.g., Z170A Gaming Pro / Tomahawk / Krait, etc.) so you can see which ports are disabled when M.2_1 is in use? That will confirm whether your drive is in the wrong slot.

### User (2025-09-19 21:01:34)

I've already reviewed the lane sharing diagram, thank you. My 10TB drive was tested in every SATA port, include 1-4. None of them worked. The BIOS date I seem to remember seeing it saying 2018, but I believe my brother ran a newer BIOS Upgrade on it back in 2023. I'm wondering now if 10tb drive is simply not supported. You still think you would see the serial number in the BIOS even if that was the case...

### Assistant (2025-09-19 21:01:34)

Got it — thanks for clarifying. Since you’ve already tested every SATA port (including 1–4) and the CD drive proves that the SATA controller itself is working, that narrows things down quite a bit. Let’s dissect the situation carefully:

---

## 1. Should the BIOS at least see the drive?
Yes — normally, even if a drive is “too big” for the OS or the controller to address properly, the BIOS should still show *something* (serial/model string, even if the reported size is wrong). The fact that you see **nothing at all** suggests one of these:
- The drive isn’t initializing/handshaking correctly with the controller.  
- The BIOS firmware truly doesn’t understand that class of device (unlikely for 2018+ firmware).  
- Or the drive’s firmware is failing at power-on (can still spin, but no ID).  

---

## 2. Large drive support on Z170
- Z170-era chipsets support **48-bit LBA**, which goes up to 128 PiB — so in theory, a 10 TB drive is no problem.  
- Practical limits are usually in the OS, not BIOS. Ubuntu 24 with GPT has no issue with 10 TB either.  
- That means: the chipset should support it, but the BIOS must properly enumerate. MSI did release updates to improve compatibility with >4 TB and >8 TB drives.  

If you’re on a BIOS from 2018, you might not have the *very last* revisions MSI published. Some boards got late updates in 2019–2020 specifically to handle high-capacity drives better.

---

## 3. Drive health possibility
Even if you believe the drive is good:  
- A failing SATA HDD can spin up but not return ID/SMART info. In that case, BIOS shows nothing — just like what you’re seeing.  
- The definitive test: connect it with a **USB-to-SATA adapter/dock** or another PC. If it enumerates and you can read SMART data, the drive is fine. If not, the drive firmware or controller board may be the culprit.

---

## 4. Testing with controller settings
- Double-check **SATA mode** is AHCI, not RAID.  
- Try toggling “Hot Plug” for the port in BIOS — sometimes this forces a re-init.  
- If the BIOS has a “Rescan Storage” option, try that with only the 10 TB drive connected.  
- Also worth: booting a Linux live USB, run `lsblk` and `dmesg | grep -i sata` — sometimes Linux sees the device even if BIOS does not list it in the setup screen.

---

## 5. Most likely scenarios now
1. **Drive firmware issue / dead PCB** → would explain no ID string in BIOS on any port.  
2. **BIOS revision not fully supporting 10 TB enumeration** → though I’d still expect *something* (misreported capacity).  
3. **Rare incompatibility between that specific 10 TB model and the Z170 SATA controller** → possible, though less common.

---

🔎 **Concrete next test**:  
- Plug the 10 TB into *another machine* (or USB dock). If it enumerates, the issue is MSI BIOS/compatibility. If it doesn’t enumerate at all, the drive itself is at fault.  

---

Do you want me to check **which exact MSI Z170A BIOS version added large-drive compatibility** so you can confirm whether your board has it? That would rule out “firmware too old” vs. “drive dead.”

### User (2025-09-19 21:04:06)

I don't have any other machine available or any other way of testing the drive. And I purchased it online 2 years ago so it will not be valid for return. A 10tb HDD costed me around $500. I have never used it until today. This is awful.

### Assistant (2025-09-19 21:04:06)

I hear you — that’s brutal. Dropping $500 on a big HDD, keeping it safe for two years, and then having it not even enumerate when you finally plug it in… that’s a gut punch. Let’s slow down and make sure we’ve really exhausted every possible angle before writing it off, because even if it can’t be returned, you might still be able to revive it.  

---

## Why it *might* still be salvageable
1. **BIOS blind spots:**  
   Even if the BIOS setup screen doesn’t list the disk, the Linux kernel is often better at probing devices. It’s worth booting Ubuntu live, opening a terminal, and running:
   ```bash
   dmesg | grep -i sata
   lsblk
   sudo fdisk -l
   ```
   If the drive is electrically alive, Linux may show *something* (e.g. a device `/dev/sdX`) even when the BIOS does not.

2. **Drive PCB fuses / power-up quirks:**  
   Some large-capacity HDDs (especially Helium-filled ones like 10TB+ models) are picky about spin-up current. A weak SATA power rail can let them spin but fail to fully initialize.  
   - If your PSU has multiple SATA power chains, try a different one.  
   - If the drive is on the same chain as the CD drive, try giving it its own connector.  

3. **Firmware freezes:**  
   A HDD can appear “dead” if the onboard firmware gets stuck during initialization. Sometimes leaving it powered for 5–10 minutes and then rebooting helps it complete initialization at least once.

---

## If the drive really is bad
- **Manufacturer warranty:** Most enterprise-class 10TB drives (Seagate Exos, WD Gold, HGST Ultrastar) carry **5-year warranties**, even if you bought it through a reseller. Even if you can’t return it to the seller, you may be able to RMA directly with the manufacturer.  
- **Data recovery shops:** Even if not under warranty, many HDD repair shops can swap the PCB or reflash firmware for ~$100–200. If the platters are fine, it’s usually recoverable.  
- **DIY PCB swap:** If it’s a common model, you can sometimes buy a matching donor PCB for $40–60 and swap it (sometimes requires firmware ROM transfer).

---

## What I’d suggest right now
1. Try powering the system with **only the 10TB drive attached** (no CD, no M.2, nothing else) → see if BIOS reacts differently.  
2. Boot Ubuntu live and check `dmesg` after plugging only the HDD.  
3. Note the **exact model number** of the HDD (printed on the label). With that, I can check if it has a manufacturer warranty you could still claim.  

---

👉 Do you want to give me the model number off the label? That way I can check if you might still have a manufacturer RMA path, even though the store return window is long gone.

