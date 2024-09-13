pub trait NanoClock {
    fn nano_time(&self) -> i64;
}

pub struct SystemNanoClock;

impl NanoClock for SystemNanoClock {
    fn nano_time(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64
    }
}
