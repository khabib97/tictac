mod app;
mod game;
mod tui;

fn main() {
    if let Err(err) = tui::run() {
        eprintln!("Terminal error: {err}");
        std::process::exit(1);
    }
}
