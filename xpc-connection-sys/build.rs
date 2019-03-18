use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::Builder::default()
        .header("wrapper.h")
        .rustfmt_bindings(true)
        .whitelist_function("dispatch_queue_create")
        .whitelist_function("xpc.*")
        .whitelist_var("xpc.*")
        .whitelist_var("_xpc.*")
        .whitelist_var("XPC.*")
        .whitelist_type("uuid_t")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
