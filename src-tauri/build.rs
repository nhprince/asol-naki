fn main() {
    tauri_build::build();

    // Windows: request Administrator elevation at the manifest level.
    //
    // Why: smartctl (and the future WMI SYSTEM queries) CANNOT read drive
    // SMART data as a normal user — \\.\PhysicalDriveN requires the
    // elevated token. Ground truth (ProBook round 4): right-click →
    // "Run as administrator" did NOT show a UAC prompt and the app still
    // failed, because the default manifest declares requestedExecutionLevel
    // = asInvoker, so Windows never elevates regardless of shell choice.
    //
    // With `requireAdministrator`, Windows MUST show the UAC consent dialog
    // on every launch — the behavior every user expects from a diagnostic
    // tool like CrystalDiskInfo / HWiNFO.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges xmlns="urn:schemas-microsoft-com:asm.v3">
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#;

        let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR unset");
        let manifest_path = std::path::Path::new(&out_dir).join("app.manifest");
        std::fs::write(&manifest_path, manifest).expect("write manifest");

        println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
            manifest_path.display()
        );
    }

    // Rerun when the build config or env changes so the manifest stays in sync.
    println!("cargo:rerun-if-changed=build.rs");
}
