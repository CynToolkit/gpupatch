#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::*;

#[napi]
pub fn hello_from_gpupatch() -> String {
    "Hello from GPUPatch core!".to_owned()
}
