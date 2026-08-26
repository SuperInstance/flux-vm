//! flux-conformance — golden-corpus conformance harness for the flux-vm
//! implementation family. Documents divergences; does not fix semantics.

pub mod corpus;
pub mod differ;
pub mod ir;
pub mod opmap;
pub mod report;
pub mod runners;

// vm/flux_vm.rs is a standalone file outside every crate in the workspace
// (the root manifest is a virtual workspace). Compiling it as a module here
// is the only way any CI job has ever executed it. Its own lint debt is
// silenced here rather than by editing their file.
#[allow(dead_code, unused_variables, clippy::all)]
#[path = "../../vm/flux_vm.rs"]
pub mod vmrs_src;
