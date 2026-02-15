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

        unsafe {
            *(OUTBOX_VA_ADDR as *mut MsgOf<UserActor>) = 282133;
        }

        self.target.send_page().await;

        Ok(())
    }
}