use super::logger;
pub fn log_manager(message: &str) {
    if let Err(e) = logger::Logger::log("~/.R-touch/logs/r-touch.log", &message) {
        eprintln!("Error logging the action. Error: {e}");
        std::process::exit(1);
    }
}
