mod boot;
pub(crate) mod cpu;
pub mod device;
pub(crate) mod irq;
mod kernel;
pub(crate) mod mm;
pub(crate) mod timer;

use alloc::fmt;
use core::fmt::Write;
use log::info;

pub(crate) fn before_all_init() {
    enable_common_cpu_features();
    device::serial::init();
    boot::init();
}

pub(crate) fn after_all_init() {
    irq::init();
    device::serial::callback_init();
    kernel::acpi::init();
    if kernel::xapic::has_apic() {
        kernel::ioapic::init();
        kernel::xapic::init();
    } else {
        info!("No apic exists, using pic instead");
        kernel::pic::enable();
    }
    timer::init();
    // Some driver like serial may use PIC
    kernel::pic::init();
}

pub(crate) fn interrupts_ack() {
    kernel::pic::ack();
    kernel::xapic::ack();
}

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            device::serial::send(c);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
  ($fmt: literal $(, $($arg: tt)+)?) => {
    $crate::arch::x86::print(format_args!($fmt $(, $($arg)+)?))
  }
}

#[macro_export]
macro_rules! println {
  ($fmt: literal $(, $($arg: tt)+)?) => {
    $crate::arch::x86::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
  }
}

fn enable_common_cpu_features() {
    use x86_64::registers::{control::Cr4Flags, model_specific::EferFlags, xcontrol::XCr0Flags};
    let mut cr4 = x86_64::registers::control::Cr4::read();
    cr4 |= Cr4Flags::FSGSBASE | Cr4Flags::OSXSAVE | Cr4Flags::OSFXSR | Cr4Flags::OSXMMEXCPT_ENABLE;
    unsafe {
        x86_64::registers::control::Cr4::write(cr4);
    }

    let mut xcr0 = x86_64::registers::xcontrol::XCr0::read();
    xcr0 |= XCr0Flags::AVX | XCr0Flags::SSE;
    unsafe {
        x86_64::registers::xcontrol::XCr0::write(xcr0);
    }

    unsafe {
        // enable non-executable page protection
        x86_64::registers::model_specific::Efer::update(|efer| {
            *efer |= EferFlags::NO_EXECUTE_ENABLE;
        });
    }
}
