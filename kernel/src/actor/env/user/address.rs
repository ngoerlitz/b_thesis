use crate::actor::env::user::environment::UserEnvironment;
use crate::isr::Svc;
use crate::{kprintln, uprintln};
use crate::platform::aarch64::cpu::current_el;
use core::fmt::{Debug, Display};
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
        if current_el() == "EL1" {
            panic!("Can't use the UserAddress::send in EL1")
        }

        uprintln!("UserAddress::send");

        async move {
            let msg_ptr = &message as *const A::Message as usize;

            unsafe {
                core::arch::asm!(
                    "svc #{svc}",
                    svc = const Svc::PrintMsg as u16,
                    in("x0") 3,     // syscall ID
                    in("x1") 0,     // target actor ID
                    in("x2") msg_ptr,
                    options(nostack)
                )
            }

            Ok(())
        }
    }
}
