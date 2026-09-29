fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut r = winresource::WindowsResource::new();
        r.set_icon_with_id("assets/icon.ico", "1");
        r.set("ProductName", "Rust Desktop Icons");
        r.compile().expect("resource compilation failed");
    }
}
