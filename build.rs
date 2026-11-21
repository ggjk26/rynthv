use std::{env, fs, path::PathBuf};

fn main() {
    // Opt-in, local-only model bundling. No third-party weights are checked
    // into the repository or downloaded by CI.
    println!("cargo::rerun-if-env-changed=RYNTHV_DURATION_DNNI");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo build output directory"));
    let destination = out.join("rynthv-duration.dnni");
    if let Some(source) = env::var_os("RYNTHV_DURATION_DNNI") {
        let source = PathBuf::from(source);
        println!("cargo::rerun-if-changed={}", source.display());
        let data = fs::read(&source).expect("cannot read RYNTHV_DURATION_DNNI");
        assert!(
            (0x149..=64 * 1024 * 1024).contains(&data.len())
                && data.starts_with(&[0xff, 0, 0xca, 0x7f, 3, 0, 0, 0]),
            "RYNTHV_DURATION_DNNI must point to a DNNI v3 file <= 64 MiB"
        );
        fs::write(&destination, data).expect("cannot stage local duration model");
    } else {
        fs::write(&destination, []).expect("cannot stage empty duration placeholder");
    }
    println!("cargo::rerun-if-changed=assets/branding/app-icon.ico");
    println!("cargo::rerun-if-changed=assets/branding/app-icon.rc");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let target = env::var("TARGET").expect("Cargo must provide TARGET");
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"));
    let resource = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo build output directory"))
        .join("app-icon.res");
    let mut compiler = find_msvc_tools::find(&target, "rc.exe")
        .expect("building the Windows app icon requires the Windows SDK resource compiler");
    let output = compiler
        .current_dir(root)
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg("assets/branding/app-icon.rc")
        .output()
        .expect("could not run the Windows SDK resource compiler");
    assert!(
        output.status.success(),
        "app icon resource compilation failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // Link the resource only into the editor, not the synthesis command-line tool.
    println!("cargo::rustc-link-arg-bin=rynthv={}", resource.display());
}
