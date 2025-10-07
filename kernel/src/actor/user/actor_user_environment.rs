use zcene_core::actor::{
    Actor, ActorCommonHandleContext, ActorEnvironment, ActorMessage, ActorMessageChannelAddress,
};

pub struct ActorUserEnvironment;

impl ActorEnvironment for ActorUserEnvironment {
    // TODO: This needs to be changed to our actor_isolation_address -> in order to facilitate copying / moving data on send.
    type Address<A>
        = ActorMessageChannelAddress<A, Self>
    where
        A: Actor<Self>;

    type CreateContext = ();
    type HandleContext<M>
        = ActorCommonHandleContext<M>
    where
        M: ActorMessage;

    type DestroyContext = ();
}
