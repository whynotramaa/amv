fn main() {
    #[cfg(feature = "desktop")]
    // Rust's +crt-static already selects the complete CRT; Tauri's separate
    // override would disable libucrt and conflict with cargo-xwin.
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new().static_vc_runtime(false)),
    )
    .expect("build desktop resources");
}
