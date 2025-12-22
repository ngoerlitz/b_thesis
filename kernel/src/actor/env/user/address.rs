use crate::actor::env::user::environment::UserEnvironment;
use core::marker::PhantomData;
use kernel_derive::Constructor;
use zcene_core::actor::{Actor, ActorAddress, ActorFuture, ActorMessageSender, ActorSendError};

#[derive(Constructor)]
pub struct UserAddress<A: Actor<UserEnvironment>> {
    descriptor: usize,
    marker: PhantomData<A::Message>,
}

impl<A: Actor<UserEnvironment>> Clone for UserAddress<A> {
    fn clone(&self) -> Self {
        Self::new(self.descriptor, self.marker)
    }
}

impl<A: Actor<UserEnvironment>> ActorAddress<A, UserEnvironment> for UserAddress<A> {}

impl<A: Actor<UserEnvironment>> ActorMessageSender<A::Message> for UserAddress<A> {
    fn send(&self, message: A::Message) -> impl ActorFuture<'_, Result<(), ActorSendError>> {
        // TODO

        async move { Ok(()) }
    }
}
