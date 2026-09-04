fn main() {
    // Leave Tauri's default manifest as-is. The product decision is:
    // app runs AS A NORMAL USER; individual privileged ops (smartctl against
    // \\.\PhysicalDriveN) elevate via ShellExecute runas on the child
    // process (see storage.rs). A requireAdministrator app-level manifest
    // would force a UAC prompt on every launch AND make the Tauri E2E
    // driver unable to spawn the app from an unelevated test harness.
    tauri_build::build();
}
