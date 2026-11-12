
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistedInodeKind {
    Regular { sector_start: u32, byte_len: usize },
    Symlink { target: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MvmPersistedInode {
    pub mode: u32,
    pub kind: PersistedInodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedCowSector {
    pub id: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MvmPersistedCow {
    pub inodes: BTreeMap<String, MvmPersistedInode>,
    pub dirs: BTreeSet<String>,
    pub cow_sectors: Vec<PersistedCowSector>,
    pub dirty_sectors: BTreeSet<u32>,
    pub next_sector: u32,
    pub next_pid: i64,
}

impl MvmSandbox {
    pub fn export_persistent_cow(&self) -> MvmPersistedCow {
        MvmPersistedCow {
            inodes: self
                .inodes
                .iter()
                .map(|(path, inode)| {
                    let kind = match &inode.kind {
                        InodeKind::Regular { sector_start, byte_len } => PersistedInodeKind::Regular {
                            sector_start: *sector_start,
                            byte_len: *byte_len,
                        },
                        InodeKind::Symlink { target } => PersistedInodeKind::Symlink { target: target.clone() },
                    };
                    (path.clone(), MvmPersistedInode { mode: inode.mode, kind })
                })
                .collect(),
            dirs: self.dirs.clone(),
            cow_sectors: self
                .cow_sectors
                .iter()
                .map(|(id, bytes)| PersistedCowSector { id: *id, bytes: bytes.clone() })
                .collect(),
            dirty_sectors: self.dirty_sectors.clone(),
            next_sector: self.next_sector,
            next_pid: self.next_pid,
        }
    }

    pub fn import_persistent_cow(&mut self, snapshot: &MvmPersistedCow) {
        self.inodes = snapshot
            .inodes
            .iter()
            .map(|(path, inode)| {
                let kind = match &inode.kind {
                    PersistedInodeKind::Regular { sector_start, byte_len } => InodeKind::Regular {
                        sector_start: *sector_start,
                        byte_len: *byte_len,
                    },
                    PersistedInodeKind::Symlink { target } => InodeKind::Symlink { target: target.clone() },
                };
                (path.clone(), GuestInode { mode: inode.mode, kind })
            })
            .collect();
        self.dirs = snapshot.dirs.clone();
        self.cow_sectors = snapshot
            .cow_sectors
            .iter()
            .map(|sector| (sector.id, sector.bytes.clone()))
            .collect();
        self.dirty_sectors = snapshot.dirty_sectors.clone();
        self.next_sector = snapshot.next_sector;
        self.next_pid = snapshot.next_pid;
    }
}
