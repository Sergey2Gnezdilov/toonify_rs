fn main() {
    // This tells Cargo to re-run this build script if the Python version changes
    println!("cargo:rerun-if-env-changed=PYTHON_SYS_EXECUTABLE");
    
    // Generate Python bindings
    pyo3_build_config::add_extension_module_link_args();
}
