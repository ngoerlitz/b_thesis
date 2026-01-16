use crate::actor::env::user::environment::UserEnvironment;
use crate::isr::SvcType;
use crate::{kprintln, uprintln};
use crate::platform::aarch64::cpu::current_el;
use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use kernel_derive::Constructor;
use zcene_core::actor::{Actor, ActorAddress, ActorFuture, ActorMessageSender, ActorSendError};

#[derive(Constructor)]
pub struct UserViewAddress<A: Actor<UserEnvironment>> {
    target_actor_id: u64,
    marker: PhantomData<A::Message>,
}

impl<A: Actor<UserEnvironment>> Clone for UserViewAddress<A> {
    fn clone(&self) -> Self {
        Self::new(self.target_actor_id, self.marker)
    }
}

impl<A: Actor<UserEnvironment>> ActorAddress<A, UserEnvironment> for UserViewAddress<A> {}

impl<A: Actor<UserEnvironment>> ActorMessageSender<A::Message> for UserViewAddress<A> {
    fn send(&self, message: A::Message) -> impl ActorFuture<'_, Result<(), ActorSendError>> {
        uprintln!("UserAddress::send");

        async move {
            let msg_ptr = &message as *const A::Message as usize;

            unsafe {
                core::arch::asm!(
                    "svc #{svc}",
                    svc = const SvcType::SendMsg as u16,
                    in("x0") self.target_actor_id,        // target actor ID
                    in("x1") msg_ptr,
                    options(nostack),
                    clobber_abi("C")
                );
            }

            Ok(())
        }
    }
}
