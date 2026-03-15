use alloc::boxed::Box;
use alloc::vec;
use core::marker::PhantomData;
use zcene_core::actor::ActorEnvironment;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::address::UserViewAddress;
use crate::receiver::ReceivingActor;
use crate::startActor::StartActor;

#[inline(always)]
pub fn register_tests() {
    // TODO
    
    let root_env = RootEnvironment::get();

    let receiver = root_env.spawn_user(
        ReceivingActor::default(),
        vec![]
    ).unwrap();

    root_env.spawn_user(
        StartActor::new(UserViewAddress::new(0, PhantomData)),
        vec![Box::new(receiver)]
    ).unwrap();
}