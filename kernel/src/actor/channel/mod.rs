pub mod pt_message;
pub mod pt_channel_address;
pub mod pt_channel_sender;
pub mod pt_channel_receiver;
pub mod pt_message_channel;

pub const OUTBOX_VA_ADDR: u64 = 0x80000000; // @ 2 GB
pub const INBOX_VA_ADDR: u64 = 0xC0000000; // @ 3 GB