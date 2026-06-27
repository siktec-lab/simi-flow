// Build script for the optional `nodejs` (napi-rs v3) binding.
//
// napi-rs v3 requires `napi_build::setup()` to wire up Node-API symbol
// resolution at build time. It only runs when the `nodejs` feature is enabled;
// for every other build (the default library, `python`, etc.) this
// is a no-op, so the core crate stays free of any Node toolchain dependency.
fn main() {
    #[cfg(feature = "nodejs")]
    napi_build::setup();
}
