use std::{net::TcpStream, thread};

use gtk::{
    gio::ApplicationFlags, glib::ExitCode, prelude::*, Application, ApplicationWindow, Grid, Label,
    WindowPosition,
};
use sd_lib::{Mode, ADDRESS};

fn main() -> ExitCode {
    thread::spawn(daemon_communication);

    let application = Application::new(None, ApplicationFlags::FLAGS_NONE);

    application.connect_activate(|app| {
        let grid = Grid::new();

        grid.set_column_homogeneous(false);

        let level = &new_label("Level");
        level.set_width_request(100);

        let application = &new_label("Application");
        application.set_width_request(1300);

        let message = &new_label("Message");
        message.set_width_request(520);

        grid.attach(level, 0, 0, 1, 1);
        grid.attach(application, 1, 0, 1, 1);
        grid.attach(message, 2, 0, 1, 1);

        grid.set_width_request(1920);

        ApplicationWindow::builder()
            .application(app)
            .title("Status Daemon")
            .window_position(WindowPosition::None)
            .decorated(false)
            .child(&grid)
            .build()
            .show_all();
    });

    application.run()
}

fn new_label(content: &str) -> Label {
    Label::new(Some(content))
}

fn daemon_communication() {
    let mut stream = TcpStream::connect(ADDRESS).unwrap();

    Mode::NewClient.transmit(&mut stream).unwrap();
}
