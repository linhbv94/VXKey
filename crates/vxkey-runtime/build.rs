fn main() {
    println!("cargo:rerun-if-env-changed=VXKEY_SIGNER_SHA1");
    println!("cargo:rerun-if-changed=native/macos_trust.c");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let sdk = std::process::Command::new("xcrun")
        .arg("--show-sdk-path")
        .output()
        .expect("macOS SDK");
    assert!(sdk.status.success());
    let sdk = String::from_utf8(sdk.stdout).unwrap();
    assert!(
        std::process::Command::new("clang")
            .args([
                "-c",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-isysroot",
                sdk.trim(),
                "native/macos_trust.c",
                "-o"
            ])
            .arg(out.join("macos_trust.o"))
            .status()
            .unwrap()
            .success()
    );
    assert!(
        std::process::Command::new("ar")
            .arg("crs")
            .arg(out.join("libvxkey_trust.a"))
            .arg(out.join("macos_trust.o"))
            .status()
            .unwrap()
            .success()
    );
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=vxkey_trust");
    for framework in ["Security", "CoreFoundation", "Carbon"] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
}
