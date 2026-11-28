use std::{env, fs, path::PathBuf};

fn main() {
    // A short revision is useful for support diagnostics, but packaging must
    // not depend on Git being installed or on a checkout containing .git.
    let revision = env::var("GITHUB_SHA")
        .ok()
        .filter(|value| value.len() >= 8 && value.chars().all(|c| c.is_ascii_hexdigit()))
        .map(|value| value[..8].to_owned())
        .or_else(|| {
            std::process::Command::new("git")
                .args(["rev-parse", "--short=8", "HEAD"])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|value| value.trim().to_owned())
                .filter(|value| value.len() >= 7 && value.chars().all(|c| c.is_ascii_hexdigit()))
        });
    if let Some(revision) = revision {
        println!("cargo::rustc-env=RYNTHV_BUILD_REV={revision}");
    }
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
    // App icons are optional packaging metadata. ARM64 and GNU/CI targets
    // can lack rc.exe even when the Rust target itself is supported. The
    // absence of a Windows SDK must not stop synthesis/editor compilation.
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        println!("cargo::warning=Windows resource compiler is MSVC-only; build without embedded executable icon");
        return;
    }
    let Some(mut compiler) = find_msvc_tools::find(&target, "rc.exe") else {
        println!("cargo::warning=Windows SDK rc.exe not found for {target}; skipping embedded icon");
        return;
    };
    let output = compiler
        .current_dir(root)
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg("assets/branding/app-icon.rc")
        .output();
    match output {
        Ok(result) if result.status.success() => {
            // Only the GUI executable needs the Windows icon.
            println!("cargo::rustc-link-arg-bin=rynthv={}", resource.display());
        }
        Ok(result) => println!(
            "cargo::warning=Windows icon resource compilation failed ({}): {}; building without embedded icon",
            result.status,
            String::from_utf8_lossy(&result.stderr).trim(),
        ),
        Err(error) => println!(
            "cargo::warning=Windows icon resource compiler unavailable ({error}); building without embedded icon"
        ),
    }
}
