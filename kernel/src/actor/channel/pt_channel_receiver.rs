use async_channel::Receiver;
use zcene_core::future::runtime::FutureRuntimeHandler;
use zcene_core::actor::ActorMessage;
use kernel_derive::Constructor;
use crate::actor::channel::pt_message::PtMessage;

#[derive(Constructor)]
pub struct PtActorMessageChannelReceiver<M, H>
where
    M: ActorMessage,
    H: FutureRuntimeHandler
{
    receiver: Receiver<PtMessage<M, H>>
}

impl<M, H> PtActorMessageChannelReceiver<M, H>
where
    M: ActorMessage,
    H: FutureRuntimeHandler
{
    pub async fn receive(&self) -> Option<PtMessage<M, H>> {
        self.receiver.recv().await.ok()
    }
}

