sixtyfps::include_modules!();

fn main() {
    let ui = AppWindow::new();

    let ui_handle = ui.as_weak();
    ui.on_request_increase_value(move |v| {
        let ui = ui_handle.unwrap();
        ui.set_counter(ui.get_counter() + v);
    });

    ui.run();
}
