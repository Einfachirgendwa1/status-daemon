use gtk::{
    gio::ApplicationFlags, glib::ExitCode, prelude::*, Application, ApplicationWindow,
    WindowPosition,
};

fn main() -> ExitCode {
    let application = Application::new(None, ApplicationFlags::FLAGS_NONE);

    application.connect_activate(|app| {
        ApplicationWindow::builder()
            .application(app)
            .title("Status Daemon")
            .window_position(WindowPosition::None)
            .decorated(false)
            .build()
            .show_all();
    });

    application.run()
}
