use log::{debug, error, info, trace, warn};
use sd_api::{close_connection, RecommendedLogger};

fn main() {
    sd_api::init();
    RecommendedLogger::use_this().unwrap();

    error!("This is an error.");
    warn!("This is a warning.");
    info!("This is an info.");
    debug!("This is a debug message.");
    trace!("This is a trace.");

    close_connection(0);
}
