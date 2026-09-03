fn main() {
    // Windows: request Administrator elevation via Tauri's OFFICIAL
    // manifest override (WindowsAttributes::app_manifest), NOT manual
    // cargo link flags — we tried /MANIFEST:EMBED in build.rs and it hit
    // LNK1327 "execution level doesn't match manifest snippets" because
    // Tauri/tao ALSO embed a default manifest. This is the sanctioned path
    // (tauri-build docs: example shows exactly requireAdministrator).
    let windows = tauri_build::WindowsAttributes::new().app_manifest(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges xmlns="urn:schemas-microsoft-com:asm.v3">
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#,
    );
    let attrs = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attrs).expect("failed to run tauri build script");
}
