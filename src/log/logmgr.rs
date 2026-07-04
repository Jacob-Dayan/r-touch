use super::logger;
#[cfg(target_family = "windows")]
const OS_ROOT: &str = "C:\\Users\\Public";
#[cfg(target_family = "unix")]
const OS_ROOT: &str = "/var/tmp";
#[cfg(not(any(target_family = "windows", target_family = "unix")))]
const OS_ROOT: &str = ".";

pub fn success_log(message: &str) {
    let mut path = dirs_next::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from(OS_ROOT));

    path = path.join("R-touch").join("logs").join("r-touch.log");

    if let Err(e) = logger::Logger::log(path.to_str().unwrap(), &message) {
        eprintln!("Error logging the action. Error: {e}");
        std::process::exit(1);
    }
}

pub fn error_log(message: &str) {
    let mut path = dirs_next::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from(OS_ROOT));
    path = path
        .join("R-touch")
        .join("logs")
        .join("crashes")
        .join("r-touch_err.log");

    if let Err(e) = logger::Logger::log(path.to_str().unwrap(), &message) {
        eprintln!("Error logging the failure. Error: {e}");
        std::process::exit(1);
    }
}
