// Embeds the app icon into mellon.exe on Windows.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/mellon.ico")
            .set("ProductName", "Mellon")
            .set("FileDescription", "Mellon");
        res.compile().expect("embed Windows icon");
    }
}
