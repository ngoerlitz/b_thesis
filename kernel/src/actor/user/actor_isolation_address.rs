use crate::actor::user::actor_user_environment::ActorUserEnvironment;
use core::marker::PhantomData;
use zcene_core::actor::{Actor, ActorFuture, ActorMessageSender, ActorSendError};

pub struct ActorUserAddress<A>
where
    A: Actor<ActorUserEnvironment>,
{
    descriptor: usize,
    marker: PhantomData<A::Message>,
}

impl<A> ActorUserAddress<A>
where
    A: Actor<ActorUserEnvironment>,
{
    pub fn new(descriptor: usize) -> Self {
        Self {
            descriptor,
            marker: PhantomData,
        }
    }
}

impl<A> Clone for ActorUserAddress<A>
where
    A: Actor<ActorUserEnvironment>,
{
    fn clone(&self) -> Self {
        Self::new(self.descriptor)
    }
}

impl<A> ActorMessageSender<A::Message> for ActorUserAddress<A>
where
    A: Actor<ActorUserEnvironment>,
{
    fn send(&self, message: A::Message) -> impl ActorFuture<'_, Result<(), ActorSendError>> {
        async move {
            todo!();

            Ok(())
        }
    }
}
