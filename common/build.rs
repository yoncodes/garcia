use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=Config.toml");
    if env::var("PROFILE").as_deref() != Ok("release") {
        return;
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR is set"));
    let release_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("Cargo OUT_DIR has a profile directory");
    fs::copy("Config.toml", release_dir.join("config.toml"))
        .expect("copy common/Config.toml beside release binaries");
}
