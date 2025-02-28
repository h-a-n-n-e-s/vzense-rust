fn main() {
    #[cfg(feature = "run-bindgen")]
    {
        // create bindings for two APIs
        let headers = ["include/Vzense_api_560.h", "include/Scepter_api.h"];
        let binding_file_names = ["dcam560.rs", "scepter.rs"];

        for i in 0..2 {
            let bindings = bindgen::Builder::default()
                // The input header we would like to generate bindings for.
                //.header("include/Vzense_api_560.h")
                .header(headers[i])
                .wrap_unsafe_ops(true)
                // derive defaults if possible
                .derive_default(true)
                // Tell cargo to invalidate the built crate whenever any of the
                // included header files changed.
                .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
                // Finish the builder and generate the bindings.
                .generate()
                .expect("Unable to generate bindings");

            let bindings_dir = std::env::current_dir().unwrap().join("bindings");
            let bindings_file = bindings_dir.join(binding_file_names[i]); // "bindings.rs");
            bindings
                .write_to_file(bindings_file)
                .expect("Couldn't write bindings!");
        }
    }
}
