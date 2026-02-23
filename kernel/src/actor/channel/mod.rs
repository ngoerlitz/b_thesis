pub mod pt_message;
pub mod pt_channel_address;
pub mod pt_channel_sender;
pub mod pt_channel_receiver;
pub mod pt_message_channel;

pub const OUTBOX_VA_ADDR: u64 = 0xC400_0000;
pub const INBOX_VA_ADDR: u64 =  0xC000_0000; // @ 2 GB