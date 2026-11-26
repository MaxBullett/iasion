//! Cargo does not rebuild when only migration files change (ADR-0033, consequences).

fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
