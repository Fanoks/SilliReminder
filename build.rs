use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let assets_dir = manifest_dir.join("assets");
    println!("cargo:rerun-if-changed=build.rs");

    // --- Embed EXE icon on Windows (File Explorer icon, taskbar grouping icon, etc.)
    #[cfg(windows)]
    {
        let ico = assets_dir.join("icon.ico");
        println!("cargo:rerun-if-changed={}", ico.display());

        if ico.exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon(ico.to_string_lossy().as_ref());
            res.compile().expect("failed to compile Windows resources");
        }
    }

    // --- Embed PNG bytes for egui window icon (used by NativeOptions.viewport.with_icon)
    let png = assets_dir.join("icon.png");
    println!("cargo:rerun-if-changed={}", png.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let generated = out_dir.join("embedded_icon_png.rs");

    if png.exists() {
        let out_png = out_dir.join("icon.png");
        // Keep it simple: always overwrite.
        let _ = fs::create_dir_all(&out_dir);
        fs::copy(&png, &out_png).expect("failed to copy icon.png into OUT_DIR");
        fs::write(
            &generated,
            "pub const ICON_PNG: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/icon.png\"));\n",
        )
        .expect("failed to write embedded_icon_png.rs");
    } else {
        fs::write(&generated, "pub const ICON_PNG: &[u8] = &[];\n")
            .expect("failed to write embedded_icon_png.rs");
    }
}
