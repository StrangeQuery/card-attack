//! Every machine in segments/*/machines, built natively: the same source as its CARDIAC tape, with the
//! built-ins reading and punching cards on the host (build.rs lists them; see segments/shared/cardiac.rs and cardiac_host::native).
include!(concat!(env!("OUT_DIR"), "/machines.rs"));
