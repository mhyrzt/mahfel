//! Boxed future alias for dyn-compatible ports.
//!
//! Application ports must be usable behind a trait object at the composition
//! boundary. A trait method returning `impl Future` (RPITIT) is not
//! dyn-compatible, so every port method returns [`BoxFuture`] instead. The
//! boxing cost is paid uniformly at the port boundary, and the transport holds
//! one `Arc<dyn Port>` per capability rather than a generic parameter per
//! store.

use std::{future::Future, pin::Pin};

/// A `Send` future boxed for dyn dispatch.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
