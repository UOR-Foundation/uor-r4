# Hard drive for Plex server

- **Conversation ID:** `69013ae9-e6f4-8325-8393-4d9a1343ebc7`
- **Date:** 2025-10-28

---

### User (2025-10-28 17:51:40)

I only have a 1tb nvme disk on my ubuntu server - it will run out of space eventually as a media server - what additional hard drive should I purchase and how would I add it to my plex stack

### Assistant (2025-10-28 17:51:41)

search("I only have a 1tb nvme disk on my ubuntu server - it will run out of space eventually as a media server - what additional hard drive should I purchase and how would I add it to my plex stack")

### Assistant (2025-10-28 17:51:45)

Great—since you already have a 1 TB NVMe on your Ubuntu server for your entity["software","Plex Media Server",0] stack, adding more storage makes perfect sense for media (which tends to grow rapidly). I’ll walk through **two parts**: (1) picking a good additional hard drive, and (2) how to integrate it into your Plex/Ubuntu setup. Since you’re highly mathematically and technically minded, I’ll lean a little deeper into best-practice and caveats, not just “plug and play”.

---

## 1. What kind of hard drive to buy

Here are the key criteria + recommended models:

### Key criteria

- **Capacity**: Media (especially 4K video) eats space. Some sources suggest 4–6 TB is the “sweet spot” for straightforward home media server use. citeturn0search5turn0search9  
- **Drive type & workload rating**: Look for drives built for “always on” or NAS style use (24×7 access, multi-user). For example, NAS drives from WD Red/Red Plus or Seagate IronWolf series. citeturn0search22turn0search17turn0search0  
- **Recording technology – avoid SMR if possible**: Some media server folks warn against SMR (“shingled magnetic recording”) drives because under certain workloads (large sequential writes / rebuilds) their performance or reliability may suffer. citeturn0search24turn0search9  
- **Interface**: SATA 3.5″ internal drives are cost-effective and typical for media libraries. If you have a lot of physical space/bays in your server, go internal. USB external also works but may have some limitations/complexities.  
- **Budget vs future-proofing**: If you anticipate a large library (many terabytes), consider buying big now (8–16 TB) rather than constantly upgrading smaller drives. citeturn0search2turn0search6  

### Recommended drives

Here are some good options (you can pick based on how much capacity you want / your budget).

products{"selections":[["turn0product7","Seagate IronWolf Pro 16 TB"],["turn0product3","WD Red Pro 16 TB"],["turn0product5","Toshiba N300 Pro 8 TB"],["turn0product1","Seagate IronWolf 4 TB"],["turn0product0","WD Red Plus 4 TB"],["turn0product10","Seagate BarraCuda 8 TB"],["turn0product8","WD Black 8 TB"],["turn0product11","WD My Book Desktop External 4 TB"]],"tags":["top large NAS drive","high end NAS pro","mid NAS drive","budget NAS 4TB","entry NAS 4TB","desktop large general","desktop performance large","external easy add"]}

And a little commentary:

- product_entity["turn0product7","Seagate IronWolf Pro 16 TB"]: Large capacity, CMR, built for NAS/24×7 usage. Great if you’re planning “many many” TB of media.  
- product_entity["turn0product3","WD Red Pro 16 TB"]: Similar high capacity NAS-grade; another top tier.  
- product_entity["turn0product5","Toshiba N300 Pro 8 TB"]: “Mid” sized – good if you’re not yet at “tens of TB” but want room to grow.  
- product_entity["turn0product1","Seagate IronWolf 4 TB"]: Smaller capacity but still NAS-oriented; useful if you want to add incrementally.  
- product_entity["turn0product0","WD Red Plus 4 TB"]: More budget-friendly NAS-style.  
- product_entity["turn0product10","Seagate BarraCuda 8 TB"]: Desktop class rather than pure NAS; fine for a simple home media server but maybe fewer “enterprise/NAS” reliability features.  
- product_entity["turn0product8","WD Black 8 TB"]: High performance desktop drive; if streaming usage is heavy maybe this helps, but you’ll sacrifice some of the NAS-tuned aspects.  
- product_entity["turn0product11","WD My Book Desktop External 4 TB"]: External USB solution – convenient if you don’t want to open up the server or your server has limited internal drive bays. Some caveats regarding mount/performance though.

### My recommendation given your use case

Since you are running a media server and likely anticipate growth (given how fast media libraries can balloon), I’d suggest:

- Prefer **at least 8 TB** for the additional drive if budget allows, especially if you already have a 1 TB NVMe (which likely holds OS and Plex metadata rather than the entire library).  
- Use a **NAS-grade drive** (e.g., IronWolf, WD Red) to get better longevity, vibration tolerance, and reliability for 24×7 use.  
- Internal SATA if your server supports it (which I assume it does). If you only have one bay/slot, external is okay but plan for maybe slower speeds, more setup.  
- Make sure you format/use a Linux-friendly file system (ext4, XFS, etc) rather than NTFS/ExFAT if it’s fully internal and dedicated to Ubuntu. This avoids weird permission/ownership issues.  
- Consider future redundancy/backups: If this media matters (and I assume you have lots of content you don’t want to lose), maybe plan a mirror or backup strategy (even if your immediate goal is “just add space”).

---

## 2. How to add the drive to your Ubuntu + Plex stack

Here’s a step-by-step high-level procedure you can follow and then I’ll highlight advanced things given your deep math/technical background, but we won’t go too deep into file-system or RAID modelling unless you want.

### Step-by-step

1. **Power off your server** (for internal installation) or plug in drive (for external).  
2. **Install the drive physically** (mount in bay, connect SATA + power cables) OR connect via USB if external.  
3. **Boot Ubuntu**.  
4. **Identify the drive device**. In terminal:  
   ```bash
   sudo lsblk
   sudo fdisk -l
   ```  
   You’ll see something like `/dev/sdb` or `/dev/sdc`.  
5. **Partition and format** (if blank/unused). Example:  
   ```bash
   sudo parted /dev/sdb
   # create GPT, one partition /dev/sdb1 for example
   ```
   Then format:  
   ```bash
   sudo mkfs.ext4 /dev/sdb1   # or mkfs.xfs if you prefer
   ```  
6. **Create a mount point**. For example:  
   ```bash
   sudo mkdir /mnt/media2
   ```  
7. **Mount the partition** to that mount point to test:  
   ```bash
   sudo mount /dev/sdb1 /mnt/media2
   ```  
   Check permissions:  
   ```bash
   ls -ld /mnt/media2
   ```

8. **Make the mount permanent** by editing `/etc/fstab`. But first find the UUID:  
   ```bash
   sudo blkid /dev/sdb1
   ```
   Then add a line in `/etc/fstab`, e.g.:  
   ```
   UUID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx  /mnt/media2  ext4  defaults  0  2
   ```
   (use correct filesystem if not ext4) (You’ll see similar guidance in AskUbuntu) citeturn0search4turn0search26  
9. **Adjust ownership/permissions** so your Plex server user can access the media. On Ubuntu, Plex often runs as user `plex`. Example:  
   ```bash
   sudo chown -R plex:plex /mnt/media2
   sudo chmod -R 755 /mnt/media2
   ```  
   Then ensure Plex sees the path as a library in Plex’s settings. Reddit user commentary:  
   > “just mount new drive, make it work. Create a folder, fix permissions and then link it in plex…” citeturn0search8  
10. **In Plex**: Go to Plex Media Server → Settings → Libraries → Add Library → select the media type (Movies/TV/Music) → browse to `/mnt/media2/[your folder]` → Add. Then let Plex scan.  
11. **Verify that streaming works**, that your media from the new drive appears correctly.  
12. (Optional) If you want to move existing media from the 1 TB to the new drive: copy files (e.g., `rsync -azP /old/media/path/ /mnt/media2/`), update Plex library paths, maybe remove old drive data.  
13. **Backup / safeguard**: Make sure you have a backup plan (external backup, RAID, etc) because a single drive remains a single point of failure.

### Additional thoughts (for your advanced context)

- Since you are interested in “modular rotation”, “alignment”, etc: think of your media storage expansion as part of a **scalable vector space** of storage. The new drive becomes a new “dimension” in your storage manifold. Ensuring mount point stability (via `fstab`) ensures your path space is continuous and does not jump across reboots (no “coordinate shift”).  
- If you expect to add *multiple* drives over time, you may consider logical volume management (LVM) or even ZFS/btrfs pooling so that drives can be added (or removed) while maintaining one filesystem namespace. E.g., you could create a volume group “media” and add new physical volumes as you buy them. Then `/mnt/media` remains continuous. This would align with your “recursive modular propagation” philosophy: each drive adds a new dimension of space and the aggregate remains aligned.  
- Use a file system tuned for large files and many sequential reads (since Plex often reads large video files). Ext4/XFS are excellent; ZFS gives you redundancy and snapshots (but has complexity).  
- Monitor SMART attributes, vibration, workload: Since the drive will be “always on” for streaming and possibly multiple simultaneous clients, monitor health and plan for replacement (NAS-grade drives typically have longer warranties/MTBF).  
- Think about data alignment/performance: If you have many concurrent streams or heavy I/O (multiple users transcoding), you may want drives with higher RPM or cache. But for typical home use streaming (one or two streams), even standard 5400 RPM NAS drives are “good enough”. Reddit commentary:  
   > “If you’re going to have multiple disks in a RAID array … you’ll want something like a WD Red (CMR only) or Seagate IronWolf.” citeturn0search0  
- Keep in mind that the primary bottleneck for Plex streaming is often **CPU/transcoding** rather than the drive speed (unless you have many streams or network bottlenecks). So while drive choice matters, once you have “sufficient” drive speed, you won’t see much benefit from ultra-high end unless you scale up significantly.

---

If you like, I can **tailor a specific shortlist of drives** optimized for *your server environment* (taking into account your server’s existing slots/connectors, expected growth over e.g. next 3–5 years, budget constraint) **and** provide a **detailed script** (bash) that automates the mounting, permission-setting, and Plex library addition for Ubuntu (so you just run it and it handles the setup). Would that be helpful?

