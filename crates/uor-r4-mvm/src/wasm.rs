use std::cell::RefCell;
use std::collections::BTreeMap;

use js_sys::Promise;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::{
    compile_c_to_rv64_elf, MvmPersistedCow, MvmSandbox, PersistedCowSector, PersistedInodeKind,
};

#[wasm_bindgen(inline_js = "
export function uorMvmSaveSnapshot(key, state) {
    return globalThis.__uorMvmHost.saveSnapshot(key, state);
}
export function uorMvmLoadSnapshot(key, snapshotId, digest) {
    return globalThis.__uorMvmHost.loadSnapshot(key, snapshotId, digest);
}
export function uorMvmLoadCurrentSnapshot(key) {
    return globalThis.__uorMvmHost.loadCurrentSnapshot(key);
}
")]
extern "C" {
    #[wasm_bindgen(js_name = uorMvmSaveSnapshot)]
    fn save_snapshot(key: &str, state: JsValue) -> Promise;
    #[wasm_bindgen(js_name = uorMvmLoadSnapshot)]
    fn load_snapshot(key: &str, snapshot_id: &str, digest: &str) -> Promise;
    #[wasm_bindgen(js_name = uorMvmLoadCurrentSnapshot)]
    fn load_current_snapshot(key: &str) -> Promise;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedFile {
    text: String,
    revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedSnapshot {
    version: u32,
    snapshot_id: String,
    cwd: String,
    cow_digest: String,
    files: BTreeMap<String, PersistedFile>,
    sectors: Vec<PersistedCowSector>,
    cow: MvmPersistedCow,
    digest: String,
}

type PendingCallback = Box<dyn FnOnce(JsValue) -> JsValue>;
type ThenCallback = Closure<dyn FnMut(JsValue) -> Result<JsValue, JsError>>;

thread_local! {
    static VM: RefCell<Option<MvmSandbox>> = const { RefCell::new(None) };
    static REVISIONS: RefCell<BTreeMap<String, u64>> = const { RefCell::new(BTreeMap::new()) };
    static CWD: RefCell<String> = RefCell::new("/workspace".to_string());
    static PENDING_CALLBACK: RefCell<Option<PendingCallback>> = const { RefCell::new(None) };
    static THEN_CALLBACK: RefCell<Option<ThenCallback>> = const { RefCell::new(None) };
}

fn response(id: u64, ok: bool, value: Value) -> JsValue {
    let body = if ok {
        json!({"id": id, "ok": true, "result": value})
    } else {
        json!({"id": id, "ok": false, "error": value.as_str().unwrap_or("MVM request failed")})
    };
    JsValue::from_str(&body.to_string())
}

fn resolved(id: u64, result: Result<Value, String>) -> Promise {
    match result {
        Ok(value) => Promise::resolve(&response(id, true, value)),
        Err(error) => Promise::resolve(&response(id, false, json!(error))),
    }
}

fn then(promise: Promise, callback: impl FnOnce(JsValue) -> JsValue + 'static) -> Promise {
    PENDING_CALLBACK.with(|slot| {
        *slot.borrow_mut() = Some(Box::new(callback));
    });
    THEN_CALLBACK.with(|wrapper| {
        let mut wrapper = wrapper.borrow_mut();
        if wrapper.is_none() {
            *wrapper = Some(Closure::new(|value| {
                let callback = PENDING_CALLBACK.with(|slot| slot.borrow_mut().take());
                match callback {
                    Some(callback) => Ok(callback(value)),
                    None => Err(JsError::new("MVM callback state is unavailable")),
                }
            }));
        }
        match wrapper.as_ref() {
            Some(callback) => promise.then_map(callback),
            None => resolved(
                1,
                Err("MVM callback dispatcher could not initialize".to_string()),
            ),
        }
    })
}

fn with_vm<T>(operation: impl FnOnce(&mut MvmSandbox) -> Result<T, String>) -> Result<T, String> {
    VM.with(|slot| {
        let mut borrowed = slot.borrow_mut();
        if borrowed.is_none() {
            *borrowed = Some(MvmSandbox::boot_default().map_err(|error| error.to_string())?);
        }
        match borrowed.as_mut() {
            Some(vm) => operation(vm),
            None => Err("MVM failed to initialize".to_string()),
        }
    })
}

fn status() -> Result<Value, String> {
    with_vm(|vm| {
        Ok(json!({
            "architecture": vm.architecture(),
            "guestCid": vm.guest_cid(),
            "dirtyCowSectorCount": vm.dirty_cow_sector_count(),
            "cowStateSha256": vm.cow_state_sha256(),
        }))
    })
}

fn snapshot_state(key: &str) -> Result<(String, JsValue), String> {
    if key.is_empty() || key.len() > 256 {
        return Err("sandbox key is invalid".to_string());
    }
    let state = with_vm(|vm| {
        let files = vm.export_workspace_text_files();
        let sectors = vm.export_dirty_cow_sectors();
        let cow_digest = vm.cow_state_sha256();
        let cwd = CWD.with(|cwd| cwd.borrow().clone());
        Ok((files, sectors, cow_digest, cwd))
    })?;
    let (files, sectors, cow_digest, cwd) = state;
    let file_records = REVISIONS.with(|revisions| {
        let revisions = revisions.borrow();
        files
            .into_iter()
            .map(|(path, text)| {
                let revision = revisions.get(&path).copied().unwrap_or(1);
                (path, PersistedFile { text, revision })
            })
            .collect::<BTreeMap<_, _>>()
    });
    let cow_digest = format!("sha256:{cow_digest}");
    let snapshot = PersistedSnapshot {
        version: 1,
        snapshot_id: cow_digest.clone(),
        cwd,
        cow_digest,
        files: file_records,
        sectors: sectors
            .into_iter()
            .map(|(id, bytes)| PersistedCowSector { id, bytes })
            .collect(),
        cow: with_vm(|vm| Ok(vm.export_persistent_cow()))?,
        digest: String::new(),
    };
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    let value = snapshot
        .serialize(&serializer)
        .map_err(|error| error.to_string())?;
    Ok((key.to_string(), value))
}

fn persist_current(key: &str) -> Result<Promise, String> {
    let (key, state) = snapshot_state(key)?;
    Ok(save_snapshot(&key, state))
}

fn read_persistent_file(cow: &MvmPersistedCow, path: &str) -> Option<Vec<u8>> {
    let mut current = path.to_string();
    for _ in 0..8 {
        let inode = cow.inodes.get(&current)?;
        match &inode.kind {
            PersistedInodeKind::Symlink { target } => current = target.clone(),
            PersistedInodeKind::Regular {
                sector_start,
                byte_len,
            } => {
                let mut output = Vec::with_capacity(*byte_len);
                let mut remaining = *byte_len;
                let mut sector_id = *sector_start;
                while remaining > 0 {
                    let sector = cow
                        .cow_sectors
                        .iter()
                        .find(|sector| sector.id == sector_id)?;
                    let take = remaining.min(sector.bytes.len());
                    output.extend_from_slice(&sector.bytes[..take]);
                    remaining -= take;
                    sector_id += 1;
                }
                return Some(output);
            }
        }
    }
    None
}

fn apply_snapshot(value: JsValue) -> Result<Value, String> {
    let snapshot: PersistedSnapshot = serde_wasm_bindgen::from_value(value)
        .map_err(|error| format!("invalid persisted snapshot: {error}"))?;
    if snapshot.version != 1 {
        return Err("unsupported persisted snapshot version".to_string());
    }
    let mut vm = MvmSandbox::boot_default().map_err(|error| error.to_string())?;
    vm.import_persistent_cow(&snapshot.cow);
    let restored_digest = format!("sha256:{}", vm.cow_state_sha256());
    if restored_digest != snapshot.cow_digest {
        let restored = vm.export_persistent_cow();
        let different_inode = snapshot
            .cow
            .inodes
            .iter()
            .find(|(path, inode)| restored.inodes.get(*path) != Some(inode))
            .map(|(path, _)| path);
        let dirs_equal = snapshot.cow.dirs == restored.dirs;
        let sectors_equal = snapshot.cow.cow_sectors == restored.cow_sectors;
        let full_cow_equal = restored == snapshot.cow;
        let different_file = snapshot.cow.inodes.keys().find(|path| {
            let expected = read_persistent_file(&snapshot.cow, path);
            let actual = vm.read_guest_file(path);
            expected.is_none() || actual.is_none() || expected != actual
        });
        let inode_count = snapshot.cow.inodes.len();
        let file_count = snapshot.files.len();
        let sector_count = snapshot.cow.cow_sectors.len();
        return Err(format!(
            "restored CoW digest mismatch: expected {}, got {restored_digest}; first_inode_difference={different_inode:?}; dirs_equal={dirs_equal}; sectors_equal={sectors_equal}; full_cow_equal={full_cow_equal}; different_file={different_file:?}; inode_count={inode_count}; file_count={file_count}; sector_count={sector_count}",
            snapshot.cow_digest
        ));
    }
    VM.with(|slot| *slot.borrow_mut() = Some(vm));
    REVISIONS.with(|revisions| {
        *revisions.borrow_mut() = snapshot
            .files
            .iter()
            .map(|(path, file)| (path.clone(), file.revision))
            .collect();
    });
    CWD.with(|cwd| *cwd.borrow_mut() = snapshot.cwd);
    status()
}

fn restore_current(key: &str, snapshot_id: Option<&str>, digest: Option<&str>) -> Promise {
    let promise = match (snapshot_id, digest) {
        (Some(snapshot_id), Some(digest)) => load_snapshot(key, snapshot_id, digest),
        _ => load_current_snapshot(key),
    };
    then(promise, |value| {
        if value.is_null() || value.is_undefined() {
            return response(1, true, status().unwrap_or_else(|error| json!(error)));
        }
        match apply_snapshot(value) {
            Ok(status) => response(1, true, status),
            Err(error) => response(1, false, json!(error)),
        }
    })
}

fn changed_revisions(before: &BTreeMap<String, String>, after: &BTreeMap<String, String>) {
    REVISIONS.with(|revisions| {
        let mut revisions = revisions.borrow_mut();
        for (path, text) in after {
            if before.get(path) != Some(text) {
                let next = revisions.get(path).copied().unwrap_or(0) + 1;
                revisions.insert(path.clone(), next);
            }
        }
        for path in before.keys() {
            if !after.contains_key(path) {
                revisions.remove(path);
            }
        }
    });
}

fn persist_result(id: u64, key: &str, result: Result<Value, String>) -> Promise {
    let result = match result {
        Ok(value) => value,
        Err(error) => return resolved(id, Err(error)),
    };
    let promise = match persist_current(key) {
        Ok(promise) => promise,
        Err(error) => return resolved(id, Err(error)),
    };
    then(promise, move |_| response(id, true, result))
}

fn execute(payload: &Value) -> Result<Value, String> {
    let cmd = payload
        .get("cmd")
        .and_then(Value::as_str)
        .ok_or("exec.cmd is required")?;
    if cmd.len() > 65_536 {
        return Err("command exceeds its byte bound".to_string());
    }
    let cwd = payload
        .get("cwd")
        .and_then(Value::as_str)
        .unwrap_or("/workspace");
    CWD.with(|current| *current.borrow_mut() = cwd.to_string());
    let before = with_vm(|vm| Ok(vm.export_workspace_text_files()))?;
    let outcome = with_vm(|vm| vm.exec_command(cmd, cwd).map_err(|error| error.to_string()))?;
    let after = with_vm(|vm| Ok(vm.export_workspace_text_files()))?;
    changed_revisions(&before, &after);
    Ok(json!({
        "exitCode": outcome.exit_code,
        "stdout": outcome.stdout,
        "stderr": outcome.stderr,
        "instructionsRetired": outcome.instructions_retired,
        "compiledElfMachine": outcome.compiled_elf_machine,
        "cowStateSha256": outcome.cow_state_sha256,
    }))
}

fn read_file(payload: &Value) -> Result<Value, String> {
    let path = payload
        .get("path")
        .and_then(Value::as_str)
        .ok_or("read.path is required")?;
    let text = with_vm(|vm| {
        vm.read_guest_file(path)
            .ok_or_else(|| format!("guest file not found: {path}"))
    })?;
    let text = String::from_utf8(text).map_err(|_| "guest file is not UTF-8 text".to_string())?;
    if text.len() > 1_048_576 {
        return Err("guest file exceeds its output bound".to_string());
    }
    let revision = REVISIONS.with(|revisions| revisions.borrow().get(path).copied().unwrap_or(1));
    Ok(json!({"path": path, "text": text, "revision": revision}))
}

fn write_file(payload: &Value) -> Result<Value, String> {
    let path = payload
        .get("path")
        .and_then(Value::as_str)
        .ok_or("write.path is required")?;
    let text = payload
        .get("text")
        .and_then(Value::as_str)
        .ok_or("write.text is required")?;
    if text.len() > 1_048_576 {
        return Err("guest file exceeds its byte bound".to_string());
    }
    if let Some(base_revision) = payload.get("baseRevision").and_then(Value::as_u64) {
        let current =
            REVISIONS.with(|revisions| revisions.borrow().get(path).copied().unwrap_or(1));
        if current != base_revision {
            return Err(format!(
                "stale editor save: expected revision {base_revision}, current {current}"
            ));
        }
    }
    with_vm(|vm| {
        vm.write_guest_file(path, text.as_bytes())
            .map_err(|error| error.to_string())
    })?;
    let revision = REVISIONS.with(|revisions| {
        let mut revisions = revisions.borrow_mut();
        let next = revisions.get(path).copied().unwrap_or(0) + 1;
        revisions.insert(path.to_string(), next);
        next
    });
    Ok(json!({"path": path, "revision": revision}))
}

fn compile(payload: &Value) -> Result<Value, String> {
    let source = payload
        .get("source")
        .and_then(Value::as_str)
        .ok_or("compile.source is required")?;
    if source.len() > 1_048_576 {
        return Err("C source exceeds its byte bound".to_string());
    }
    let elf = compile_c_to_rv64_elf(source).map_err(|error| error.to_string())?;
    Ok(json!({"machine": 243, "bytes": elf}))
}

fn json_bytes(value: &Value, label: &str) -> Result<Vec<u8>, String> {
    let values = value
        .as_array()
        .ok_or_else(|| format!("{label} must be a byte array"))?;
    values
        .iter()
        .map(|byte| {
            byte.as_u64()
                .filter(|byte| *byte <= u8::MAX as u64)
                .map(|byte| byte as u8)
                .ok_or_else(|| format!("{label} contains an invalid byte"))
        })
        .collect()
}

fn vsock(payload: &Value) -> Result<Value, String> {
    let action = payload
        .get("action")
        .and_then(Value::as_str)
        .ok_or("vsock.action is required")?;
    match action {
        "build" => {
            let dst_cid = payload
                .get("dstCid")
                .and_then(Value::as_u64)
                .ok_or("dstCid is required")? as u16;
            let port = payload.get("port").and_then(Value::as_u64).unwrap_or(1024) as u16;
            let spin = payload.get("spin").and_then(Value::as_u64).unwrap_or(0) as u8;
            let orbit = payload.get("orbit").and_then(Value::as_u64).unwrap_or(0) as u8;
            let bytes = json_bytes(payload.get("bytes").ok_or("bytes is required")?, "bytes")?;
            let packet = with_vm(|vm| {
                vm.build_vsock_packet(dst_cid, port, spin, orbit, &bytes)
                    .map_err(|error| error.to_string())
            })?;
            Ok(json!({"packet": packet}))
        }
        "deliver" => {
            let packet = json_bytes(payload.get("packet").ok_or("packet is required")?, "packet")?;
            with_vm(|vm| {
                vm.deliver_vsock_packet(&packet)
                    .map_err(|error| error.to_string())
            })?;
            Ok(json!({"delivered": true}))
        }
        "pop" => {
            let port = payload.get("port").and_then(Value::as_u64).unwrap_or(1024) as u16;
            let packet = with_vm(|vm| Ok(vm.pop_vsock_inbox(port)))?;
            Ok(
                json!({"packet": packet.map(|packet| json!({"srcCid": packet.header.src_cid, "dstCid": packet.header.dst_cid, "payload": packet.payload}))}),
            )
        }
        _ => Err("unsupported vsock action".to_string()),
    }
}

#[wasm_bindgen]
pub fn mvm_dispatch(request_json: &str) -> Promise {
    if request_json.len() > 2_097_152 {
        return resolved(
            1,
            Err("protocol message exceeds its byte bound".to_string()),
        );
    }
    let request: Value = match serde_json::from_str(request_json) {
        Ok(value) => value,
        Err(error) => return resolved(1, Err(format!("invalid request JSON: {error}"))),
    };
    let id = request.get("id").and_then(Value::as_u64).unwrap_or(1);
    let kind = request.get("kind").and_then(Value::as_str).unwrap_or("");
    let payload = request.get("payload").cloned().unwrap_or_else(|| json!({}));
    let key = payload
        .get("sandboxKey")
        .and_then(Value::as_str)
        .unwrap_or("");
    match kind {
        "init" => restore_current(key, None, None),
        "status" => resolved(id, status()),
        "exec" => persist_result(id, key, execute(&payload)),
        "read" => resolved(id, read_file(&payload)),
        "write" => persist_result(id, key, write_file(&payload)),
        "snapshot" => {
            let promise = match persist_current(key) {
                Ok(promise) => promise,
                Err(error) => return resolved(id, Err(error)),
            };
            then(
                promise,
                move |metadata| match serde_wasm_bindgen::from_value::<Value>(metadata) {
                    Ok(value) => response(id, true, value),
                    Err(error) => response(
                        id,
                        false,
                        json!(format!("invalid snapshot metadata: {error}")),
                    ),
                },
            )
        }
        "restore" => {
            let snapshot_id = payload.get("snapshotId").and_then(Value::as_str);
            let digest = payload.get("digest").and_then(Value::as_str);
            if snapshot_id.is_none() || digest.is_none() {
                return resolved(
                    id,
                    Err("restore requires snapshotId and digest".to_string()),
                );
            }
            let promise = load_snapshot(key, snapshot_id.unwrap_or(""), digest.unwrap_or(""));
            then(promise, move |value| match apply_snapshot(value) {
                Ok(status) => response(id, true, status),
                Err(error) => response(id, false, json!(error)),
            })
        }
        "compile" => resolved(id, compile(&payload)),
        "vsock" => resolved(id, vsock(&payload)),
        "dispose" => {
            VM.with(|slot| *slot.borrow_mut() = None);
            REVISIONS.with(|revisions| revisions.borrow_mut().clear());
            resolved(id, Ok(json!({"disposed": true})))
        }
        _ => resolved(id, Err("request kind is not allowlisted".to_string())),
    }
}
