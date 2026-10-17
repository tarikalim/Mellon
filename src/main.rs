#![forbid(unsafe_code)]

mod config;
mod mcp;
mod store;
mod theme;
mod ui;

fn main() {
    match std::env::args().nth(1).as_deref() {
        None => ui::run(),
        Some("mcp") => mcp::serve().expect("stdio closed"),
        Some(_) => {
            eprintln!(
                "usage:\n  mellon        open the window\n  mellon mcp    MCP server over stdio"
            );
            std::process::exit(2)
        }
    }
}
