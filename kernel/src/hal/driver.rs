
/// Any hardware abstraction that should be used as a driver **must** implement this trait.
/// The `Driver` trait defines a specific set of functions which enable the OS to directly
/// influence the behaviour of the device - specifically enabling and disabling. It also contains
/// a unique name describing the type of device the driver is written for.
pub(crate) trait Driver {
    const NAME: &'static str;

    fn enable(&mut self) -> Result<(), ()> {
        Ok(())
    }

    fn disable(&mut self);

    fn name(&self) -> &'static str {
        Self::NAME
    }
}