use core::arch::asm;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::environment::UserEnvironment;
use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorFuture, ActorHandleError};

#[derive(Default)]
pub struct UserActor;

pub type UserActorMessage = i32;

#[unsafe(link_section = ".user_text")]
static MESSAGE: [u8; 17] = *b"User Text Message";

#[inline(always)]
#[unsafe(link_section = ".user_text")]
fn buf_ptr_len(b: &[u8; 17]) -> (*const u8, usize) {
    (b.as_ptr(), b.len())
}

#[inline(always)]
#[unsafe(link_section = ".user_text")]
fn sys_write(buf: *const u8, len: usize) {
    unsafe {
        asm!(
        "svc #0x20",
        in("x0") buf,
        in("x1") len,
        options(nostack, preserves_flags)
        );
    }
}

impl Actor<UserEnvironment> for UserActor {
    #[unsafe(link_section = ".user_text")]
    type Message = UserActorMessage;

    #[unsafe(link_section = ".user_text")]
    async fn create<'a>(
        &'a mut self,
        context: <UserEnvironment as ActorEnvironment>::CreateContext<'a>,
    ) -> Result<(), ActorCreateError> {
        sys_write(MESSAGE.as_ptr(), MESSAGE.len());

        Ok(())
    }

    #[unsafe(link_section = ".user_text")]
    fn handle<'a>(
        &mut self,
        context: <UserEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>,
    ) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        async move {
            let (p, n) = buf_ptr_len(&MESSAGE);
            sys_write(p, n);

            Ok(())
        }
    }
}
