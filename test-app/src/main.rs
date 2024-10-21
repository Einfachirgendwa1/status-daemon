use log::{error, info, warn, LevelFilter, Log};
use sd_api::report;

struct Logger {}

impl Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        report(record);

        println!("{}: {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

fn main() {
    sd_api::init();

    log::set_logger(&Logger {}).unwrap();
    log::set_max_level(LevelFilter::Trace);

    info!("Hello, World!");
    warn!("This is a warning.");
    error!("This is a error.");

    loop {}
}
