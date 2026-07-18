//! # UOR-aligned R⁴ Tangent Space Router — facade crate
//!
//! One dependency for library users. This crate re-exports the workspace's
//! library crates:
//!
//! - [`uor_r4_core`]: pure mathematics — zeta-zero embeddings, Hopf
//!   coordinates, prime/QIMC identity layer, state metrics.
//! - [`uor_r4_router`]: the engine plus its UOR witness layer —
//!   [`UorR4Router`] state, manifold indexing, geometric generation, the
//!   routing axis (`R4Axis`), shapes, and the `UorR4RouterModel` PrismModel
//!   (wasm-bindgen surface included).
//! - [`uor_tless`]: the transformerless engine — multiplication-free,
//!   table-native, certifiable inference (compiler, runtime, certifier).
//! - [`uor_tless_bindings`]: the UOR rebase of the transformerless engine —
//!   `TlessAxis`, shapes, `UorTlessModel`, uor-addr addressing, and
//!   per-prediction `Grounded` certificates (aliased [`tless_uor`]).
//!
//! Every path the binary and previous consumers used is preserved at the
//! crate root; [`prelude`] is the ergonomic one-import surface.

pub use uor_r4_core;
pub use uor_r4_router;
pub use uor_tless;
pub use uor_tless_bindings;

pub use uor_r4_core::*;
pub use uor_r4_router::*;

pub use uor_tless_bindings as tless_uor;

/// The one-import surface for library users.
pub mod prelude {
    pub use uor_r4_core::{R4Vector, ALPHA_4, ALPHA_5};
    pub use uor_r4_router::{
        GeometricResponse, R4HostBounds, R4RoutingInput, R4RoutingOutput, RoutingData,
        UorR4Router, UorR4RouterModel, ACTIVE_ROUTER,
    };
    pub use uor_tless::runtime::OpKernel;
    pub use uor_tless_bindings as tless_uor;
}

#[cfg(test)]
mod facade_smoke_tests {
    #[test]
    fn reexport_paths_resolve() {
        let _ = core::any::type_name::<crate::UorR4Router>();
        let _ = core::any::type_name::<crate::RoutingData>();
        let _ = core::any::type_name::<crate::R4Vector>();
        let _ = core::any::type_name::<crate::UorR4RouterModel>();
        let _ = core::any::type_name::<crate::tless_uor::UorTlessModel>();
        let _ = core::any::type_name::<crate::prelude::UorR4Router>();
        assert!(crate::prelude::ALPHA_4 > 0.0);
    }
}
