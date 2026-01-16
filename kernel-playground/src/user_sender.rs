use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorMessageSender};
use kernel::actor::env::user::address::UserViewAddress;
use kernel::actor::env::user::environment::UserEnvironment;
use kernel::uprintln;
use crate::user::{UserActor};
use kernel_derive::Constructor;

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

        self.target.send("[1] Hello world, this is a message from one of your buddies! Another user actor").await;

        Ok(())
    }
}