use log::{debug, error, info, trace, warn};
use sd_api::{close_connection, use_recommended_logger};

fn main() {
    sd_api::init();
    use_recommended_logger().unwrap();

    error!("This is an error.");
    warn!("This is a warning.");
    info!("This is an info.");
    debug!("This is a debug message.");
    trace!("This is a trace.");

    close_connection(0);
}
