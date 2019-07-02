fn main() {
    let mut ui = fehui::FehUI::new();

    let mut hotload = fehui_hotload::Hotload::new("target/debug/examples", "hotload-app").unwrap(); // todo: release support
    hotload.reload(&mut ui);

    loop {
        hotload.try_reload(&mut ui);
    }
}