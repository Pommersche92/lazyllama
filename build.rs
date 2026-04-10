//! Build script for lazyllama.
//!
//! When cross-compiling for Windows, embeds `icon.ico` as the application icon
//! so Explorer and the taskbar display it correctly.
//!
//! The `.ico` file is generated from `icon.png` by `scripts/release-github.sh`
//! before the Windows cross-compile step. If the file is absent the build
//! proceeds without an icon and emits a `cargo:warning` instead of failing.

fn main() {
    // Only do anything when the *target* platform is Windows.
    // This env var is set by Cargo and reflects the target, not the host.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let ico_src = std::path::PathBuf::from(&manifest_dir).join("icon.ico");

        if !ico_src.exists() {
            println!(
                "cargo:warning=icon.ico not found; Windows exe will have no icon. \
                 Run `convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico` \
                 to generate it, or use scripts/release-github.sh which does this automatically."
            );
            return;
        }

        // Copy icon next to the .rc file so windres can resolve the relative path.
        let ico_dest = std::path::PathBuf::from(&out_dir).join("lazyllama.ico");
        std::fs::copy(&ico_src, &ico_dest).expect("failed to copy icon to OUT_DIR");

        // Write a minimal Windows resource script.
        let rc_path = std::path::PathBuf::from(&out_dir).join("lazyllama.rc");
        std::fs::write(&rc_path, "1 ICON \"lazyllama.ico\"\n")
            .expect("failed to write resource script");

        // Compile the .rc → COFF object with the mingw windres tool.
        let obj_path = std::path::PathBuf::from(&out_dir).join("lazyllama_icon.o");
        let windres = "x86_64-w64-mingw32-windres";
        let status = std::process::Command::new(windres)
            .args([
                rc_path.to_str().unwrap(),
                "-o",
                obj_path.to_str().unwrap(),
                "--target=pe-x86-64",
            ])
            // Run inside OUT_DIR so relative paths in the .rc file resolve.
            .current_dir(&out_dir)
            .status();

        match status {
            Ok(s) if s.success() => {
                // Tell the linker to include the resource object.
                println!("cargo:rustc-link-arg={}", obj_path.display());
            }
            Ok(s) => {
                println!("cargo:warning=windres exited with {s}; Windows exe will have no icon");
            }
            Err(e) => {
                println!(
                    "cargo:warning=Could not run {windres}: {e}; Windows exe will have no icon"
                );
            }
        }
    }
}
