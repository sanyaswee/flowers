fn main() {
    // Rebuild if new migrations were created
    println!("cargo:rerun-if-changed=migrations");
}
