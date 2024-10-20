use sd_api::send_test_message;

fn main() {
    sd_api::init().unwrap();

    send_test_message("Hello, World!").unwrap();
}
