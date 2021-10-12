use core::fmt;
use core::ops;
use gimli::{Register, X86};

// Match DWARF_FRAME_REGISTERS in libgcc
pub const MAX_REG_RULES: usize = 17;

#[repr(C)]
#[derive(Clone, Default)]
pub struct Context {
    pub registers: [usize; 8],
    pub ra: usize,
    pub mcxsr: usize,
    pub fcw: usize,
}

impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_struct("Context");
        for i in 0..=7 {
            fmt.field(
                X86::register_name(Register(i as _)).unwrap(),
                &self.registers[i],
            );
        }
        fmt.field("ra", &self.ra)
            .field("mcxsr", &self.mcxsr)
            .field("fcw", &self.fcw)
            .finish()
    }
}

impl ops::Index<Register> for Context {
    type Output = usize;

    fn index(&self, reg: Register) -> &usize {
        match reg {
            Register(0..=7) => &self.registers[reg.0 as usize],
            X86::RA => &self.ra,
            X86::MXCSR => &self.mcxsr,
            _ => unimplemented!(),
        }
    }
}

impl ops::IndexMut<gimli::Register> for Context {
    fn index_mut(&mut self, reg: Register) -> &mut usize {
        match reg {
            Register(0..=7) => &mut self.registers[reg.0 as usize],
            X86::RA => &mut self.ra,
            X86::MXCSR => &mut self.mcxsr,
            _ => unimplemented!(),
        }
    }
}

#[naked]
pub extern "C-unwind" fn save_context() -> Context {
    // No need to save caller-saved registers here.
    unsafe {
        asm!(
            "
            mov eax, [esp + 4]
            mov [eax + 4], ecx
            mov [eax + 8], edx
            mov [eax + 12], ebx
            mov [eax + 20], ebp
            mov [eax + 24], esi
            mov [eax + 28], edi

            /* Adjust the stack to account for the return address */
            lea edx, [esp + 4]
            mov [eax + 16], edx

            mov edx, [esp]
            mov [eax + 32], edx
            stmxcsr [eax + 36]
            fnstcw [eax + 40]
            ret 4
            ",
            options(noreturn)
        );
    }
}

#[naked]
pub unsafe extern "C" fn restore_context(ctx: &Context) -> ! {
    unsafe {
        asm!(
            "
            mov edx, [esp + 4]

            /* Restore stack */
            mov esp, [edx + 16]

            /* Restore callee-saved control registers */
            ldmxcsr [edx + 36]
            fldcw [edx + 40]

            /* Restore return address */
            mov eax, [edx + 32]
            push eax

            /*
            * Restore general-purpose registers. Non-callee-saved registers are
            * also restored because sometimes it's used to pass unwind arguments.
            */
            mov eax, [edx + 0]
            mov ecx, [edx + 4]
            mov ebx, [edx + 12]
            mov ebp, [edx + 20]
            mov esi, [edx + 24]
            mov edi, [edx + 28]

            /* EDX restored last */
            mov edx, [edx + 8]

            ret
            ",
            options(noreturn)
        );
    }
}
