# Motherboard Resets Boot Order

- **Conversation ID:** `56a7acb0-4d2f-4856-ae58-95780782c4da`
- **Date:** 2024-03-03

---

### User (2024-03-03 15:25:43)

Computer resets and motherboard seems to reset bios and data drive boots before m2

### Assistant (2024-03-03 15:26:15)

When your computer resets and the BIOS settings revert, causing the data drive to boot before the M.2 drive, it suggests an issue that could stem from various sources. Let's tackle this problem step by step:

### 1. CMOS Battery Check
The first step is to check the CMOS battery on the motherboard. This battery maintains the BIOS settings when the computer is powered off. If the battery is weak or dead, it can cause the BIOS settings to reset every time the computer is turned off.

**Action:** Open the computer case and locate the CMOS battery on the motherboard. It's usually a silver coin-shaped battery. Replace it with a new one, preferably CR2032 or equivalent, and then check if the problem persists.

### 2. BIOS Update
An outdated BIOS can have bugs or compatibility issues with hardware, especially newer components like an M.2 drive.

**Action:** Visit the motherboard manufacturer's website, find the support page for your specific motherboard model, and download the latest BIOS update. Be sure to follow the manufacturer’s instructions carefully when updating the BIOS, as incorrect updates can cause serious issues.

### 3. Boot Order Configuration
If the BIOS resets, it might not remember the preferred boot order, causing it to default to another drive.

**Action:** Access the BIOS setup utility by pressing the designated key during the boot process (often Del, F2, or Esc, depending on the motherboard). Once in the BIOS, navigate to the boot order settings and ensure the M.2 drive is set as the first boot device. Save the changes and exit the BIOS.

### 4. BIOS Settings Saving Issue
Sometimes, the issue might be with saving the BIOS settings correctly.

**Action:** After setting the boot order, ensure you're choosing the 'Save and Exit' option in the BIOS. Some BIOS versions ask for confirmation to save changes; always confirm.

### 5. Check for Faulty Hardware
A faulty M.2 drive, data drive, or even the motherboard itself can cause boot issues.

**Action:** Test the M.2 and data drives on another computer if possible, or use a known good drive as a test in your system. This can help isolate if the issue is with the drives or the motherboard.

### 6. Electrical Issues
Improper grounding or electrical shorts can cause erratic behavior, including BIOS resets.

**Action:** Ensure the motherboard is correctly installed with standoffs to prevent it from touching the case directly, which could cause shorts. Check all power connections to ensure they are secure and not damaged.

### 7. Consult the Motherboard Manual
The motherboard manual can provide specific instructions or notes about M.2 and data drive configurations, including any known issues or specific settings required for stable operation.

**Conclusion:** By following these steps methodically, you should be able to identify and resolve the issue causing the BIOS to reset and the boot order to change. If the problem persists after these steps, it might indicate a deeper hardware issue, potentially requiring professional diagnosis and repair.

