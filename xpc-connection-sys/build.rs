use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::Builder::default()
        .header("wrapper.h")
        .rustfmt_bindings(true)
        .allowlist_function("dispatch_queue_create")
        .allowlist_function("xpc.*")
        .allowlist_var("xpc.*")
        .allowlist_var("_xpc.*")
        .allowlist_var("XPC.*")
        .allowlist_type("uuid_t")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
