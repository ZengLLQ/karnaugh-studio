fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Karnaugh Studio");
        res.set("FileDescription", "Karnaugh Studio Boolean Minimizer");
        res.set("LegalCopyright", "Copyright (C) 2026 Karnaugh Studio Team");
        if let Err(e) = res.compile() {
            eprintln!("Warning: failed to compile windows resource: {e}");
        }
    }
}
