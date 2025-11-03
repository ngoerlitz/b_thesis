pub trait SystemTimer {
    fn get_value(&self) -> u64;
    fn get_frequency(&self) -> u64;
}
