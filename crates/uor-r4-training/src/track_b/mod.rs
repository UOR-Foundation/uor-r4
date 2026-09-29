//! Offline Track B conversion and distillation tools.
//!
//! The dense Candle model is an offline teacher and conversion baseline. It is
//! not the native geometric serving runtime and carries no serving-cost claim.

pub mod conversion;
pub mod generation;
pub mod harmonics;
pub mod model;
