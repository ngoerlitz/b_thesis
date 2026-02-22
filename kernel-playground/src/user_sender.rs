use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorMessageSender};
use kernel::actor::channel::OUTBOX_VA_ADDR;
use kernel::actor::env::user::address::{MsgOf, UserViewAddress};
use kernel::actor::env::user::environment::UserEnvironment;
use kernel::uprintln;
use crate::user::{UserActor};
use kernel_derive::Constructor;
use crate::receiver::ReceivingActor;

#[derive(Constructor)]
pub struct UserSender {
    target: UserViewAddress<UserActor>
}

impl Actor<UserEnvironment> for UserSender {
    #[unsafe(link_section = ".user_text")]
    type Message = ();

    #[unsafe(link_section = ".user_text")]
    async fn create<'a>(
        &'a mut self,
        context: <UserEnvironment as ActorEnvironment>::CreateContext<'a>,
    ) -> Result<(), ActorCreateError> {
        uprintln!("[1] CREATING UserSender");

        // self.target.send(512).await;

        // TODO: Sending User <-> User messages is broken:
        // [CORE: 3], UNHANDLED EXCEPTION/IRQ
        // 	ESR_EL1  = 0x000000008600000e (EC=0x21, ISS=0xe)
        // 	ELR_EL1  = 0x0000000000000001
        // 	FAR_EL1  = 0x0000000000000001
        // 	SPSR_EL1 = 0x00000000800003c5
        // or a derivative of that. 

        unsafe {
            *(OUTBOX_VA_ADDR as *mut MsgOf<UserActor>) = 282133;
        }

        self.target.send_page().await;

        unsafe {
            *(OUTBOX_VA_ADDR as *mut MsgOf<UserActor>) = 999999;
        }

        self.target.send_page().await;

        Ok(())
    }
}