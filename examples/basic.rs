use fehui;
use fehui::{Widget, Build};
use fehui_derive::Widget;
use fehui::views;
use fehui::widgets::Row;

/// Persistent state information for `App`
struct AppState {
    color: [f32; 4],
}

#[derive(Debug, Widget)]
#[widget(build, state)]
struct App {
    main_color: [f32; 4],
}

impl App {
    /// Declarative Widget
    ///
    /// `state` is optional and may be removed if not needed (remove `state` attribute).
    fn build(&self, state: &AppState, build: &mut Build) -> Row {
        Row {
            children: views![
                Row { children: views![] },
                Row { children: views![] },
            ],
        }
    }

    /// Create the state for the app widget.
    ///
    /// Do not call this from user code. The UI will internally execute this if needed.
    fn state(&self) -> AppState {
        AppState { color: self.main_color }
    }
}

fn main() {
    let mut ui = fehui::FehUI::new();

    loop {
        /// Rebuild the UI with `App` as root widget.
        ui.build(App {
            main_color: [0.2, 0.0, 0.1, 1.0],
        });
    }
}
