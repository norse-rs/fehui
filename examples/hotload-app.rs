use fehui::{Widget, Build};
use fehui_derive::Widget;
use fehui::views;
use fehui::widgets::Row;
use fehui_hotload::fehui_hook;
use fehui;

#[derive(Debug, Widget)]
#[widget(build)]
struct App {
    main_color: [f32; 4],
}

impl App {
    fn build(&self, build: &mut Build) -> Row {
        println!("build app");

        Row {
            children: views![
                Row { children: views![] },
                Row { children: views![] },
            ],
        }
    }
}

fehui_hook!(|ui: &mut fehui::FehUI| {
    ui.build(App { main_color: [0.0, 0.1, 1.0, 0.0] });
});