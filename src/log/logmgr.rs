use super::logger;
pub fn success_log(message: &str) {
    let mut path = dirs_next::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));

    path = path.join("R-touch").join("logs").join("r-touch.log");

    if let Err(e) = logger::Logger::log(path.to_str().unwrap(), &message) {
        eprintln!("Error logging the action. Error: {e}");
        std::process::exit(1);
    }
}
// Version 0.2.5
pub fn error_log(message: &str) {
    let mut path = dirs_next::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path = path
        .join("R-touch")
        .join("logs")
        .join("crashes")
        .join("r-touch_err.log");

    if let Err(e) = logger::Logger::log(path.to_str().unwrap(), &message) {
        eprintln!("Error logging the failer. Error: {e}");
        std::process::exit(1);
    }
}
