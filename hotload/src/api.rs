#[macro_export]
macro_rules! fehui_hook {
    ($load:expr) => {
        #[no_mangle]
        extern "C" fn fehui_load(ui: *mut fehui::FehUI) {
            println!("fehui :: load");
            let ui = unsafe { &mut *ui };
            $load(ui);
        }
    };
}
