use super::{digest, Index, Result};
use serde_json::Value;
use uor_r4_discrete::{device::Device, snapshot::Snapshot, Identity, ReadSpec, Spec};

pub struct Engine {
    full: Device,
    knockout: Device,
    context: usize,
    width: usize,
}

fn envelope(bytes: &[u8]) -> Result<(Value, &[u8])> {
    if bytes.get(..8) != Some(b"UORLUT01") {
        return Err("invalid D11 magic".into());
    }
    let length = usize::try_from(u64::from_le_bytes(
        bytes.get(8..16).ok_or("short D11")?.try_into()?,
    ))?;
    let end = length.checked_add(16).ok_or("header length overflow")?;
    Ok((
        serde_json::from_slice(bytes.get(16..end).ok_or("short header")?)?,
        bytes.get(end..).ok_or("short payload")?,
    ))
}

impl Engine {
    pub fn load(carrier: &[u8]) -> Result<Self> {
        let (carry_header, payload) = envelope(carrier)?;
        let reader_bytes = carry_header["reader_bytes"]
            .as_u64()
            .ok_or("unbound reader length")?;
        let parent = payload
            .get(usize::try_from(reader_bytes)?..)
            .ok_or("short carrier")?;
        if carry_header["parent_sha256"] != digest(parent) {
            return Err("parent binding differs".into());
        }
        let (header, payload) = envelope(parent)?;
        let shape = &header["shape"];
        if header["schema"] != "uor-r4.lut-discrete-trunk/1"
            || shape["read_mode"] != "relation"
            || shape["read_position"].as_u64().unwrap_or(0) != 0
        {
            return Err("device observer supports the frozen non-positional Relation trunk".into());
        }
        let padding = (64 - (parent.len() - payload.len()) % 64) % 64;
        let data = payload.get(padding..).ok_or("short parent payload")?;
        let section = |name: &str| -> Result<Vec<u32>> {
            let item = header["tables"]
                .as_array()
                .ok_or("no table inventory")?
                .iter()
                .find(|t| t["name"] == name)
                .ok_or("missing required table")?;
            let offset = usize::try_from(
                item["span"]["offset"]
                    .as_u64()
                    .ok_or("missing table offset")?,
            )?;
            let bytes =
                usize::try_from(item["span"]["bytes"].as_u64().ok_or("missing table size")?)?;
            let words = data
                .get(offset..offset.checked_add(bytes).ok_or("table overflow")?)
                .ok_or("truncated table")?;
            if bytes % 4 != 0 {
                return Err("unaligned table words".into());
            }
            Ok(words
                .chunks_exact(4)
                .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
                .collect())
        };
        let unsigned = |key: &str| -> Result<u32> {
            Ok(u32::try_from(
                shape[key].as_u64().ok_or("missing shape field")?,
            )?)
        };
        let source = |key: &str| -> Result<String> {
            Ok(header["source"][key]
                .as_str()
                .ok_or("missing source binding")?
                .to_owned())
        };
        let spec = Spec {
            vocabulary: unsigned("vocabulary")?,
            blocks: unsigned("blocks")?,
            context: unsigned("context")?,
            kept_cells: unsigned("kept_cells")?,
        };
        let full = Snapshot {
            schema: source("snapshot_schema")?,
            spec,
            read_spec: ReadSpec::Relation {
                kept: unsigned("read_kept")?,
                sweeps: unsigned("read_sweeps")?,
            },
            objective_policy: serde_json::from_value(shape["objective_policy"].clone())?,
            read_initialization: serde_json::from_value(shape["read_initialization"].clone())?,
            normalization_mode: Default::default(),
            position: None,
            address_metric: Default::default(),
            seed: unsigned("seed")?,
            epoch: shape["epoch"].as_u64().ok_or("missing epoch")?,
            table: section("discrete.table")?
                .into_iter()
                .map(|word| word as i32)
                .collect(),
            routes: section("discrete.routes")?,
            gates: section("discrete.gates")?
                .into_iter()
                .map(|word| word as i32)
                .collect(),
            operator_kappa: source("operator_kappa")?,
            composition_kappa: source("composition_kappa")?,
            kernel_sha256: source("kernel_sha256")?,
            source_sha256: source("source_sha256")?,
            geometry_sha256: source("geometry_sha256")?,
        };
        let mut knockout = full.clone();
        // Geometry's fixed identity is zero, matching with_state_transport_knockout.
        knockout.routes.fill(0);
        knockout.gates.fill(0);
        Ok(Self {
            full: Device::from_snapshot_for_qualification(full, 0)?,
            knockout: Device::from_snapshot_for_qualification(knockout, 0)?,
            context: spec.context as usize,
            width: spec.blocks as usize * 8,
        })
    }

    pub fn states(&self, prompts: &[&[u32]], knockout: bool) -> Result<Vec<Vec<i32>>> {
        let device = if knockout { &self.knockout } else { &self.full };
        let mut states = Vec::with_capacity(prompts.len());
        for chunk in prompts.chunks(128) {
            let windows: Vec<Vec<Identity>> = chunk
                .iter()
                .enumerate()
                .map(|(row, prefix)| {
                    prefix[prefix.len().saturating_sub(self.context)..]
                        .iter()
                        .enumerate()
                        .map(|(i, &token)| Identity {
                            token,
                            occurrence: i as u64,
                            record: row as u64,
                            version: 0,
                        })
                        .collect()
                })
                .collect();
            let batch = device.carry_windows(&windows)?;
            // CUDA forward's diagnostic writer target is always 0. Only the
            // pre-writer read-vector words are returned and used downstream.
            let words = device.forward(&batch, &vec![0; chunk.len()])?.vectors()?;
            if words.len() != chunk.len() * self.width {
                return Err("device state readback has invalid shape".into());
            }
            states.extend(words.chunks_exact(self.width).map(<[i32]>::to_vec));
        }
        Ok(states)
    }

    pub fn matches(
        &self,
        queries: &[Vec<i32>],
        index: &Index,
    ) -> Result<Vec<(Option<u64>, usize)>> {
        let mut labels = std::collections::BTreeMap::new();
        let catalog: Vec<_> = index
            .prototypes
            .iter()
            .map(|p| {
                let n = labels.len() as u32;
                *labels.entry(p.forms.as_slice()).or_insert(n)
            })
            .collect();
        let keys: Vec<i32> = index
            .prototypes
            .iter()
            .flat_map(|p| p.state.iter().copied())
            .collect();
        let query: Vec<i32> = queries.iter().flatten().copied().collect();
        self.full
            .question_state_matches(&query, &keys, &catalog)
            .map_err(Into::into)
    }
}
