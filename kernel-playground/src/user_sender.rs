use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorMessageSender};
use kernel::actor::channel::OUTBOX_VA_ADDR;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::address::{MsgOf, UserViewAddress};
use kernel::actor::env::user::environment::UserEnvironment;
use kernel::uprintln;
use crate::user::{UserActor};
use kernel_derive::Constructor;
use crate::benchmark_actor::BenchmarkActor;
use crate::receiver::ReceivingActor;

// #[derive(Constructor)]
// pub struct UserSender {
//     target: UserViewAddress<ReceivingActor>
// }
//
// impl Actor<UserEnvironment> for UserSender {
//     #[unsafe(link_section = ".user_text")]
//     type Message = ();
//
//     #[unsafe(link_section = ".user_text")]
//     async fn create<'a>(
//         &'a mut self,
//         context: <UserEnvironment as ActorEnvironment>::CreateContext<'a>,
//     ) -> Result<(), ActorCreateError> {
//         uprintln!("[1] CREATING UserSender");
//
//         let mut x: MsgOf<ReceivingActor> = [0; 262_144];
//
//         for i in 0..x.len() {
//             x[i] = i as u64;
//         }
//
//         self.target.send(x).await;
//
//         // self.target.send(512).await;
//
//         // unsafe {
//         //     let mut y = &mut *(OUTBOX_VA_ADDR as *mut MsgOf<ReceivingActor>);
//         //
//         //     for i in 0..y.len() {
//         //         y[i] = i as u64;
//         //     }
//         // }
//         //
//         // self.target.send_page().await;
//
//         Ok(())
//     }
// }