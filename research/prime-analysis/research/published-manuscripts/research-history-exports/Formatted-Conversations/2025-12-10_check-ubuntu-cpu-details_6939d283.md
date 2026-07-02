# Check Ubuntu CPU details

- **Conversation ID:** `6939d283-93b8-8331-a673-e22aaa59f982`
- **Date:** 2025-12-10

---

### User (2025-12-10 15:05:35)

how do I check what processor the ubuntu server is running from the terminal?

### Assistant (2025-12-10 15:05:36)

Here are the quickest and most useful commands to identify your CPU from the Ubuntu terminal:

---

### **1. Full CPU Details (best overall)**  
```bash
lscpu
```
This gives you:
- Model name  
- Architecture  
- Core count / thread count  
- Clock speeds  
- Virtualization support  
- Cache sizes  

---

### **2. Quick model name only**  
```bash
grep -m1 "model name" /proc/cpuinfo
```

---

### **3. All raw CPU info**  
```bash
cat /proc/cpuinfo
```
(Shows each core as a separate block.)

---

### **4. Hardware summary**  
```bash
sudo dmidecode -t processor
```
This gives the most BIOS-level info (brand string, socket type, etc.), but requires sudo.

---

### **5. CPU utilization + model**  
If you want usage as well as identity:
```bash
top
```
Press **1** inside `top` to show per-core usage.

---

If you want, paste the `lscpu` output here and I’ll break down exactly where your bottlenecks are—especially relevant for the GPU/NVMe lane conflict and any Plex transcoding performance.

