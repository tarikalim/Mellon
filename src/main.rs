#![forbid(unsafe_code)]

mod app;
mod config;
mod mcp;
mod store;

use eframe::egui;

const ICON: &[u8] = include_bytes!("../assets/icon.rgba");
const ICON_SIZE: u32 = 256;

fn main() -> eframe::Result<()> {
    if std::env::args().nth(1).as_deref() == Some("mcp") {
        mcp::serve().expect("stdio closed");
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("envhub")
            .with_inner_size([1100.0, 700.0])
            .with_icon(egui::IconData {
                rgba: ICON.to_vec(),
                width: ICON_SIZE,
                height: ICON_SIZE,
            }),
        ..Default::default()
    };
    eframe::run_native(
        "envhub",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new()))),
    )
}
