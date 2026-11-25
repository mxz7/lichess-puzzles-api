fn main() {
    println!("cargo:rerun-if-changed=src/db/migrations");
    println!("cargo:rerun-if-changed=sqlx.toml");
}
