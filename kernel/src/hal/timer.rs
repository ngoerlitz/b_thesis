pub trait SystemTimerDriver {
    fn now(&self) -> u64;
    fn get_frequency(&self) -> u64;
}
