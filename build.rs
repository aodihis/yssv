fn main() {
    println!("cargo:rerun-if-changed=assets/images/icon.png");

    #[cfg(target_os = "windows")]
    {
        let png = std::fs::read("assets/images/icon.png").expect("read assets/images/icon.png");

        // PNG IHDR chunk: width at bytes 16..20, height at bytes 20..24 (big-endian).
        let width = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
        let height = u32::from_be_bytes([png[20], png[21], png[22], png[23]]);
        // ICONDIRENTRY width/height bytes use 0 to mean 256.
        let w_byte = if width >= 256 { 0 } else { width as u8 };
        let h_byte = if height >= 256 { 0 } else { height as u8 };

        // Windows Vista+ accepts a PNG-compressed image directly inside an ICO container.
        let mut ico = Vec::with_capacity(22 + png.len());
        ico.extend_from_slice(&[0, 0, 1, 0, 1, 0]); // ICONDIR: reserved, type=icon, count=1
        ico.push(w_byte);
        ico.push(h_byte);
        ico.push(0); // color palette
        ico.push(0); // reserved
        ico.extend_from_slice(&[1, 0]); // color planes
        ico.extend_from_slice(&[32, 0]); // bits per pixel
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes()); // image data size
        ico.extend_from_slice(&22u32.to_le_bytes()); // offset to image data
        ico.extend_from_slice(&png);

        let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
        let ico_path = std::path::Path::new(&out_dir).join("icon.ico");
        std::fs::write(&ico_path, &ico).expect("write generated icon.ico");

        // System (not Per-Monitor-V2) DPI awareness: winit's PMv2 handling has a
        // WM_DPICHANGED feedback bug on Windows that shrinks the window each time
        // it crosses monitors with different scaling. System-aware trades crisp
        // rescaling (the window bitmap-scales until restarted) for not shrinking.
        let manifest = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">System</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>"#;

        winresource::WindowsResource::new()
            .set_icon(ico_path.to_str().expect("OUT_DIR is valid UTF-8"))
            .set_manifest(manifest)
            .compile()
            .expect("failed to embed Windows icon resource");
    }
}
