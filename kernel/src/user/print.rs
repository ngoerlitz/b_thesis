use crate::isr::Svc;
use core::arch::asm;
use core::fmt;

struct StackBuf<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> StackBuf<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, len: 0 }
    }

    fn as_ptr_len(&self) -> (*const u8, usize) {
        (self.buf.as_ptr(), self.len)
    }
}

impl<'a> fmt::Write for StackBuf<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let rem = self.buf.len().saturating_sub(self.len);
        if s.len() > rem {
            return Err(fmt::Error);
        }

        self.buf[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
        self.len += s.len();
        Ok(())
    }
}

#[unsafe(link_section = ".user_text")]
pub fn user_print(args: fmt::Arguments) {
    let mut buffer = [0u8; 512];
    let mut sb = StackBuf::new(&mut buffer);

    match fmt::write(&mut sb, args) {
        Ok(_) => {
            let (p, n) = sb.as_ptr_len();
            sys_write(p, n);
        }
        Err(_) => {
            sys_write(b"[uprintln overflow]\n".as_ptr(), 20);
        }
    }
}

#[inline(always)]
#[unsafe(link_section = ".user_text")]
fn sys_write(buf: *const u8, len: usize) {
    unsafe {
        asm!(
        "svc #{svc}",
        svc = const Svc::PrintMsg as u16,
        in("x0") buf,
        in("x1") len,
        options(nostack, preserves_flags)
        );
    }
}
