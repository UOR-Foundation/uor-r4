use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt;

pub const MAX_DIMENSION: usize = 256;
pub const MAX_GRID: usize = 16;
pub const Q12_SCALE: i32 = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionError {
    InvalidDimensions,
    InvalidGrid,
    LengthMismatch { expected: usize, actual: usize },
    InvalidCanonical(&'static str),
    HashMismatch,
}

impl fmt::Display for VisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => {
                write!(f, "dimensions must be between 1 and {MAX_DIMENSION}")
            }
            Self::InvalidGrid => write!(
                f,
                "grid must be between 1 and {MAX_GRID} and no larger than either dimension"
            ),
            Self::LengthMismatch { expected, actual } => write!(
                f,
                "RGBA length mismatch: expected {expected} bytes, got {actual}"
            ),
            Self::InvalidCanonical(message) => write!(f, "invalid R4 image: {message}"),
            Self::HashMismatch => write!(f, "R4 image source hash does not match"),
        }
    }
}

impl std::error::Error for VisionError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImagePatch {
    pub row: usize,
    pub column: usize,
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
    pub mean_rgb: [u8; 3],
    pub luma: u8,
    pub chroma_rg: i16,
    pub chroma_by: i16,
    pub horizontal_edge: u8,
    pub vertical_edge: u8,
    pub q12: [i32; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImageDecomposition {
    pub schema: &'static str,
    pub width: usize,
    pub height: usize,
    pub grid: usize,
    pub source_sha256: String,
    pub patches: Vec<ImagePatch>,
}

impl ImageDecomposition {
    pub fn canonical_prompt(&self) -> String {
        let values = self
            .patches
            .iter()
            .flat_map(|patch| patch.q12)
            .collect::<Vec<_>>();
        canonical_from_values(
            self.width,
            self.height,
            self.grid,
            &self.source_sha256,
            &values,
        )
    }
}

fn canonical_from_values(
    width: usize,
    height: usize,
    grid: usize,
    hash: &str,
    values: &[i32],
) -> String {
    use std::fmt::Write;
    let mut output = format!("R4IMG1;sha256={hash};size={width}x{height};grid={grid};q12=");
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        let _ = write!(output, "{value}");
    }
    output
}

fn composite(channel: u8, alpha: u8) -> Result<u64, VisionError> {
    let product = u64::from(channel)
        .checked_mul(u64::from(alpha))
        .ok_or(VisionError::InvalidDimensions)?;
    product
        .checked_add(127)
        .map(|value| value / 255)
        .ok_or(VisionError::InvalidDimensions)
}

fn pixel_luma(rgba: &[u8], width: usize, x: usize, y: usize) -> Result<u64, VisionError> {
    let pixel = y
        .checked_mul(width)
        .and_then(|row| row.checked_add(x))
        .ok_or(VisionError::InvalidDimensions)?;
    let index = pixel.checked_mul(4).ok_or(VisionError::InvalidDimensions)?;
    let alpha = *rgba.get(index + 3).ok_or(VisionError::LengthMismatch {
        expected: index + 4,
        actual: rgba.len(),
    })?;
    let r = composite(
        *rgba.get(index).ok_or(VisionError::LengthMismatch {
            expected: index + 4,
            actual: rgba.len(),
        })?,
        alpha,
    )?;
    let g = composite(
        *rgba.get(index + 1).ok_or(VisionError::LengthMismatch {
            expected: index + 4,
            actual: rgba.len(),
        })?,
        alpha,
    )?;
    let b = composite(
        *rgba.get(index + 2).ok_or(VisionError::LengthMismatch {
            expected: index + 4,
            actual: rgba.len(),
        })?,
        alpha,
    )?;
    r.checked_mul(77)
        .and_then(|value| g.checked_mul(150).and_then(|g| value.checked_add(g)))
        .and_then(|value| b.checked_mul(29).and_then(|b| value.checked_add(b)))
        .and_then(|value| value.checked_add(128))
        .map(|value| value / 256)
        .ok_or(VisionError::InvalidDimensions)
}

fn rounded_mean(sum: u64, count: u64) -> Result<u64, VisionError> {
    if count == 0 {
        return Err(VisionError::InvalidDimensions);
    }
    sum.checked_add(count / 2)
        .map(|value| value / count)
        .ok_or(VisionError::InvalidDimensions)
}

fn bounded_mul(left: usize, right: usize) -> Result<usize, VisionError> {
    left.checked_mul(right)
        .ok_or(VisionError::InvalidDimensions)
}

pub fn decompose_rgba(
    width: usize,
    height: usize,
    rgba: &[u8],
    grid: usize,
) -> Result<ImageDecomposition, VisionError> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(VisionError::InvalidDimensions);
    }
    if grid == 0 || grid > MAX_GRID || grid > width || grid > height {
        return Err(VisionError::InvalidGrid);
    }
    let expected = bounded_mul(bounded_mul(width, height)?, 4)?;
    if rgba.len() != expected {
        return Err(VisionError::LengthMismatch {
            expected,
            actual: rgba.len(),
        });
    }

    let mut hasher = Sha256::new();
    hasher.update(b"R4IMG1\0");
    for value in [width, height, grid] {
        let value = u32::try_from(value).map_err(|_| VisionError::InvalidDimensions)?;
        hasher.update(value.to_le_bytes());
    }
    hasher.update(rgba);
    let source_sha256 = hex::encode(hasher.finalize());
    let mut patches = Vec::new();

    for row in 0..grid {
        let y0 = bounded_mul(row, height)? / grid;
        let y1 = bounded_mul(row + 1, height)? / grid;
        for column in 0..grid {
            let x0 = bounded_mul(column, width)? / grid;
            let x1 = bounded_mul(column + 1, width)? / grid;
            let mut sums = [0_u64; 3];
            let mut luma_sum = 0_u64;
            let mut count = 0_u64;
            let mut horizontal_sum = 0_u64;
            let mut vertical_sum = 0_u64;
            let mut horizontal_pairs = 0_u64;
            let mut vertical_pairs = 0_u64;

            for y in y0..y1 {
                for x in x0..x1 {
                    let pixel = y
                        .checked_mul(width)
                        .and_then(|value| value.checked_add(x))
                        .ok_or(VisionError::InvalidDimensions)?;
                    let index = bounded_mul(pixel, 4)?;
                    let alpha = rgba[index + 3];
                    for (channel, sum) in sums.iter_mut().enumerate() {
                        let value = composite(rgba[index + channel], alpha)?;
                        *sum = sum
                            .checked_add(value)
                            .ok_or(VisionError::InvalidDimensions)?;
                    }
                    let luma = pixel_luma(rgba, width, x, y)?;
                    luma_sum = luma_sum
                        .checked_add(luma)
                        .ok_or(VisionError::InvalidDimensions)?;
                    count = count.checked_add(1).ok_or(VisionError::InvalidDimensions)?;
                    if x + 1 < x1 {
                        let right = pixel_luma(rgba, width, x + 1, y)?;
                        horizontal_sum = horizontal_sum
                            .checked_add(luma.abs_diff(right))
                            .ok_or(VisionError::InvalidDimensions)?;
                        horizontal_pairs = horizontal_pairs
                            .checked_add(1)
                            .ok_or(VisionError::InvalidDimensions)?;
                    }
                    if y + 1 < y1 {
                        let below = pixel_luma(rgba, width, x, y + 1)?;
                        vertical_sum = vertical_sum
                            .checked_add(luma.abs_diff(below))
                            .ok_or(VisionError::InvalidDimensions)?;
                        vertical_pairs = vertical_pairs
                            .checked_add(1)
                            .ok_or(VisionError::InvalidDimensions)?;
                    }
                }
            }

            let mean_rgb = [
                u8::try_from(rounded_mean(sums[0], count)?)
                    .map_err(|_| VisionError::InvalidDimensions)?,
                u8::try_from(rounded_mean(sums[1], count)?)
                    .map_err(|_| VisionError::InvalidDimensions)?,
                u8::try_from(rounded_mean(sums[2], count)?)
                    .map_err(|_| VisionError::InvalidDimensions)?,
            ];
            let luma = u8::try_from(rounded_mean(luma_sum, count)?)
                .map_err(|_| VisionError::InvalidDimensions)?;
            let horizontal_edge = if horizontal_pairs == 0 {
                0
            } else {
                u8::try_from(rounded_mean(horizontal_sum, horizontal_pairs)?)
                    .map_err(|_| VisionError::InvalidDimensions)?
            };
            let vertical_edge = if vertical_pairs == 0 {
                0
            } else {
                u8::try_from(rounded_mean(vertical_sum, vertical_pairs)?)
                    .map_err(|_| VisionError::InvalidDimensions)?
            };
            let chroma_rg = i16::from(mean_rgb[0]) - i16::from(mean_rgb[1]);
            let chroma_by = i16::from(mean_rgb[2]) - i16::from(luma);
            let edge_strength = horizontal_edge.max(vertical_edge);
            let q12 = [
                i32::from(luma)
                    .checked_mul(Q12_SCALE)
                    .ok_or(VisionError::InvalidDimensions)?,
                i32::from(chroma_rg)
                    .checked_mul(Q12_SCALE)
                    .ok_or(VisionError::InvalidDimensions)?,
                i32::from(chroma_by)
                    .checked_mul(Q12_SCALE)
                    .ok_or(VisionError::InvalidDimensions)?,
                i32::from(edge_strength)
                    .checked_mul(Q12_SCALE)
                    .ok_or(VisionError::InvalidDimensions)?,
            ];
            patches.push(ImagePatch {
                row,
                column,
                x0,
                y0,
                x1,
                y1,
                mean_rgb,
                luma,
                chroma_rg,
                chroma_by,
                horizontal_edge,
                vertical_edge,
                q12,
            });
        }
    }

    Ok(ImageDecomposition {
        schema: "uor-r4.image/1",
        width,
        height,
        grid,
        source_sha256,
        patches,
    })
}

pub fn validate_canonical_prompt(prompt: &str, expected_sha256: &str) -> Result<(), VisionError> {
    if expected_sha256.len() != 64
        || !expected_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(VisionError::HashMismatch);
    }
    let (hash, rest) = prompt
        .strip_prefix("R4IMG1;sha256=")
        .ok_or(VisionError::InvalidCanonical("missing R4IMG1 header"))?
        .split_once(";size=")
        .ok_or(VisionError::InvalidCanonical("missing size"))?;
    if hash != expected_sha256 {
        return Err(VisionError::HashMismatch);
    }
    let (size, rest) = rest
        .split_once(";grid=")
        .ok_or(VisionError::InvalidCanonical("missing grid"))?;
    let (grid, values) = rest
        .split_once(";q12=")
        .ok_or(VisionError::InvalidCanonical("missing Q12 values"))?;
    let (width, height) = size
        .split_once('x')
        .ok_or(VisionError::InvalidCanonical("invalid dimensions"))?;
    let width = width
        .parse::<usize>()
        .map_err(|_| VisionError::InvalidCanonical("invalid width"))?;
    let height = height
        .parse::<usize>()
        .map_err(|_| VisionError::InvalidCanonical("invalid height"))?;
    let grid = grid
        .parse::<usize>()
        .map_err(|_| VisionError::InvalidCanonical("invalid grid"))?;
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(VisionError::InvalidCanonical("dimensions out of bounds"));
    }
    if grid == 0 || grid > MAX_GRID || grid > width || grid > height {
        return Err(VisionError::InvalidCanonical("grid out of bounds"));
    }
    if values.is_empty() {
        return Err(VisionError::InvalidCanonical("empty Q12 values"));
    }
    let values = values
        .split(',')
        .map(|value| {
            value
                .parse::<i32>()
                .map_err(|_| VisionError::InvalidCanonical("invalid Q12 value"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expected = bounded_mul(bounded_mul(grid, grid)?, 4)?;
    if values.len() != expected {
        return Err(VisionError::InvalidCanonical("wrong Q12 value count"));
    }
    for (index, value) in values.iter().enumerate() {
        let valid = match index % 4 {
            0 | 3 => (0..=4080).contains(value),
            _ => (-4080..=4080).contains(value),
        };
        if !valid {
            return Err(VisionError::InvalidCanonical("Q12 value out of bounds"));
        }
    }
    if canonical_from_values(width, height, grid, hash, &values) != prompt {
        return Err(VisionError::InvalidCanonical("non-canonical serialization"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_HASH: &str = "78d46b99b22a762639ee0c3ec19b4b8e886b2a616ae9f72b21972a3338cb6e59";
    const ONE_CANONICAL: &str = "R4IMG1;sha256=78d46b99b22a762639ee0c3ec19b4b8e886b2a616ae9f72b21972a3338cb6e59;size=1x1;grid=1;q12=304,-128,144,0";

    #[test]
    fn one_by_one_has_exact_patch_and_canonical_bytes() {
        let image = decompose_rgba(1, 1, &[12, 20, 28, 255], 1).unwrap();
        assert_eq!(image.source_sha256, ONE_HASH);
        assert_eq!(image.patches.len(), 1);
        let patch = &image.patches[0];
        assert_eq!((patch.row, patch.column), (0, 0));
        assert_eq!((patch.x0, patch.y0, patch.x1, patch.y1), (0, 0, 1, 1));
        assert_eq!(patch.mean_rgb, [12, 20, 28]);
        assert_eq!(patch.luma, 19);
        assert_eq!((patch.chroma_rg, patch.chroma_by), (-8, 9));
        assert_eq!((patch.horizontal_edge, patch.vertical_edge), (0, 0));
        assert_eq!(patch.q12, [304, -128, 144, 0]);
        assert_eq!(image.canonical_prompt(), ONE_CANONICAL);
    }

    #[test]
    fn uniform_odd_dimensions_have_complete_positioned_patches() {
        let rgba = (0..9)
            .flat_map(|_| [100, 150, 200, 255])
            .collect::<Vec<_>>();
        let image = decompose_rgba(3, 3, &rgba, 2).unwrap();
        assert_eq!(image.patches.len(), 4);
        let rectangles = image
            .patches
            .iter()
            .map(|patch| {
                (
                    patch.row,
                    patch.column,
                    patch.x0,
                    patch.y0,
                    patch.x1,
                    patch.y1,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            rectangles,
            vec![
                (0, 0, 0, 0, 1, 1),
                (0, 1, 1, 0, 3, 1),
                (1, 0, 0, 1, 1, 3),
                (1, 1, 1, 1, 3, 3),
            ]
        );
        for patch in &image.patches {
            assert_eq!(patch.mean_rgb, [100, 150, 200]);
            assert_eq!(patch.luma, 141);
            assert_eq!((patch.chroma_rg, patch.chroma_by), (-50, 59));
            assert_eq!(patch.q12, [2256, -800, 944, 0]);
        }
    }

    #[test]
    fn asymmetric_patch_has_rounded_means_chroma_and_edges() {
        let image = decompose_rgba(
            2,
            2,
            &[
                0, 0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 255, 0, 255, 0, 255,
            ],
            1,
        )
        .unwrap();
        let patch = &image.patches[0];
        assert_eq!(patch.mean_rgb, [128, 128, 64]);
        assert_eq!(patch.luma, 120);
        assert_eq!((patch.chroma_rg, patch.chroma_by), (0, -56));
        assert_eq!((patch.horizontal_edge, patch.vertical_edge), (164, 92));
        assert_eq!(patch.q12, [1920, 0, -896, 2624]);
    }

    #[test]
    fn horizontal_edge_is_directional() {
        let image = decompose_rgba(
            2,
            2,
            &[
                0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
            ],
            1,
        )
        .unwrap();
        assert_eq!(
            (
                image.patches[0].horizontal_edge,
                image.patches[0].vertical_edge
            ),
            (255, 0)
        );
    }

    #[test]
    fn vertical_edge_is_directional() {
        let image = decompose_rgba(
            2,
            2,
            &[
                0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            ],
            1,
        )
        .unwrap();
        assert_eq!(
            (
                image.patches[0].horizontal_edge,
                image.patches[0].vertical_edge
            ),
            (0, 255)
        );
    }

    #[test]
    fn alpha_is_composited_over_black_with_integer_rounding() {
        let image = decompose_rgba(1, 1, &[200, 100, 50, 128], 1).unwrap();
        let patch = &image.patches[0];
        assert_eq!(patch.mean_rgb, [100, 50, 25]);
        assert_eq!(patch.luma, 62);
        assert_eq!((patch.chroma_rg, patch.chroma_by), (50, -37));
        assert_eq!(patch.q12, [992, 800, -592, 0]);
    }

    #[test]
    fn decomposition_is_deterministic_and_pixel_changes_change_hash_and_prompt() {
        let first = decompose_rgba(1, 1, &[12, 20, 28, 255], 1).unwrap();
        let repeated = decompose_rgba(1, 1, &[12, 20, 28, 255], 1).unwrap();
        assert_eq!(first, repeated);
        let changed = decompose_rgba(1, 1, &[13, 20, 28, 255], 1).unwrap();
        assert_ne!(changed.source_sha256, first.source_sha256);
        assert_ne!(changed.canonical_prompt(), first.canonical_prompt());
    }

    #[test]
    fn canonical_validation_accepts_only_exact_bounded_serialization() {
        validate_canonical_prompt(ONE_CANONICAL, ONE_HASH).unwrap();
        for invalid in [
            "a red square",
            "R4IMG1;sha256=78d46b99b22a762639ee0c3ec19b4b8e886b2a616ae9f72b21972a3338cb6e59;size=1x1;grid=1;q12=304,-128,144",
            "R4IMG1;sha256=78d46b99b22a762639ee0c3ec19b4b8e886b2a616ae9f72b21972a3338cb6e59;size=257x1;grid=1;q12=304,-128,144,0",
        ] {
            assert!(validate_canonical_prompt(invalid, ONE_HASH).is_err(), "{invalid}");
        }
        assert!(validate_canonical_prompt(&(ONE_CANONICAL.to_owned() + "\n"), ONE_HASH).is_err());
        assert!(validate_canonical_prompt(ONE_CANONICAL, ONE_HASH).is_ok());
        assert!(validate_canonical_prompt(ONE_CANONICAL, &"0".repeat(64)).is_err());
    }

    #[test]
    fn invalid_dimensions_grid_lengths_and_overflow_are_refused() {
        assert!(decompose_rgba(0, 1, &[], 1).is_err());
        assert!(decompose_rgba(1, 0, &[], 1).is_err());
        assert!(decompose_rgba(1, 1, &[0; 4], 0).is_err());
        assert!(decompose_rgba(1, 1, &[0; 4], 2).is_err());
        assert!(decompose_rgba(257, 1, &[], 1).is_err());
        assert!(decompose_rgba(1, 257, &[], 1).is_err());
        assert!(decompose_rgba(16, 16, &[0; 1024], 17).is_err());
        assert!(decompose_rgba(1, 1, &[0; 3], 1).is_err());
        assert!(decompose_rgba(1, 1, &[0; 5], 1).is_err());
        assert!(decompose_rgba(usize::MAX, 2, &[], 1).is_err());
    }
}
