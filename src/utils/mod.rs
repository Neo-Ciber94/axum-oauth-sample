use std::time::Duration;

pub fn now() -> Duration {
    use std::time::SystemTime;
    let now = SystemTime::now();
    now.elapsed().expect("Failed to get current time")
}
