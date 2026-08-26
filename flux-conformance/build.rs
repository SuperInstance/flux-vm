// Build the two C implementations (hardened core + monitor) into a static
// library linked into this crate, so the harness can run them like any other
// implementation. Mirrors the repo CI flags (-std=c11 -Wall -Wextra -Werror).
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_src = manifest.parent().unwrap().join("src");
    let csrc = manifest.join("csrc");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let cc = env::var("CC").unwrap_or_else(|_| "cc".into());

    let units = [
        repo_src.join("flux_runtime_arm.c"),
        csrc.join("monitor_shim.c"), // includes flux_monitor_arm.c directly
    ];

    let mut objs: Vec<PathBuf> = Vec::new();
    for src in &units {
        let obj = out_dir.join(format!("{}.o", src.file_stem().unwrap().to_str().unwrap()));
        let status = Command::new(&cc)
            .args([
                "-c", "-O2", "-g", "-std=c11", "-Wall", "-Wextra", "-Werror", "-Isrc",
            ])
            .arg("-I")
            .arg(&repo_src)
            .arg("-I")
            .arg(&csrc)
            .arg(src)
            .arg("-o")
            .arg(&obj)
            .status()
            .expect("failed to spawn C compiler");
        assert!(status.success(), "C compile failed for {}", src.display());
        objs.push(obj);
    }

    let lib = out_dir.join("libfluxconf.a");
    let _ = std::fs::remove_file(&lib);
    let status = Command::new("ar")
        .arg("rcs")
        .arg(&lib)
        .args(&objs)
        .status()
        .expect("failed to spawn ar");
    assert!(status.success(), "ar failed");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=fluxconf");
    for src in &units {
        println!("cargo:rerun-if-changed={}", src.display());
    }
    println!(
        "cargo:rerun-if-changed={}",
        repo_src.join("flux_monitor_arm.c").display()
    );
    let _ = Path::new("x").exists();
}
