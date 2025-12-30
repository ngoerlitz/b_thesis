use crate::actor::env::user::address::UserAddress;
use crate::actor::env::user::ctx::UserEnvironmentHandleCtx;
use core::fmt::Debug;
use zcene_core::actor::{Actor, ActorEnvironment, ActorMessage};

pub struct UserEnvironment;

impl ActorEnvironment for UserEnvironment {
    type Address<A: Actor<Self>> = UserAddress<A>;
    type CreateContext<'a> = ();
    type HandleContext<'a, M: ActorMessage> = UserEnvironmentHandleCtx<M>;
    type DestroyContext<'a> = ();
}
