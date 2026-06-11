//! Scoped hardware denormal (subnormal) flushing for the audio callback.
//!
//! Subnormal floats are orders of magnitude slower on most CPUs, and feedback
//! DSP (delay lines, reverbs, filter memories) naturally decays into the
//! subnormal range. [`DenormalGuard`] flips the CPU's flush-to-zero behavior
//! on for the duration of a scope — typically the whole audio callback — and
//! restores the previous register state on drop.
//!
//! Per-architecture behavior:
//! - `x86_64`: sets the FTZ (flush-to-zero) and DAZ (denormals-are-zero) bits
//!   in MXCSR via the SSE control-register intrinsics.
//! - `aarch64`: sets the FZ bit in FPCR via inline assembly.
//! - other targets: no-op.
//!
//! The guard is thread-local in effect (FP control registers are per-core,
//! saved per-thread by the OS), so it deliberately does not implement `Send`
//! or `Sync`: construct it on the thread whose math it should affect.
//!
//! Kernels still call [`crate::flush_denormal`] defensively on their state:
//! the software flush also clears vanishing-but-normal feedback tails far
//! above the subnormal boundary, and keeps behavior identical on targets
//! where the hardware guard is a no-op.

use core::marker::PhantomData;

/// MXCSR flush-to-zero bit (x86_64).
#[cfg(target_arch = "x86_64")]
const MXCSR_FLUSH_TO_ZERO: u32 = 1 << 15;

/// MXCSR denormals-are-zero bit (x86_64).
#[cfg(target_arch = "x86_64")]
const MXCSR_DENORMALS_ARE_ZERO: u32 = 1 << 6;

/// FPCR flush-to-zero bit (aarch64).
#[cfg(target_arch = "aarch64")]
const FPCR_FLUSH_TO_ZERO: u64 = 1 << 24;

/// RAII guard that enables hardware flush-to-zero for the current thread and
/// restores the previous floating-point control state when dropped.
///
/// Construct one at the top of the audio callback:
///
/// ```
/// use signal_dsp::DenormalGuard;
///
/// let _guard = DenormalGuard::new();
/// // ... process the block; subnormals now flush to zero in hardware ...
/// // previous FP control state is restored when `_guard` drops
/// ```
///
/// On targets other than `x86_64` and `aarch64` the guard is a no-op.
#[derive(Debug)]
pub struct DenormalGuard {
    /// MXCSR value captured before the guard enabled FTZ/DAZ.
    #[cfg(target_arch = "x86_64")]
    saved_mxcsr: u32,
    /// FPCR value captured before the guard enabled FZ.
    #[cfg(target_arch = "aarch64")]
    saved_fpcr: u64,
    /// Pins the guard to the constructing thread (`!Send`/`!Sync`): the FP
    /// control registers it manipulates are per-thread state.
    _not_send_sync: PhantomData<*const ()>,
}

impl DenormalGuard {
    /// Enable hardware flush-to-zero, remembering the previous control state.
    #[must_use = "the guard restores the FP control state when dropped; bind it to a variable"]
    pub fn new() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            // SAFETY: `_mm_getcsr`/`_mm_setcsr` only read and write the MXCSR
            // control register of the current thread. SSE is a baseline
            // feature of the x86_64 ABI, so the instructions are always
            // available. Setting FTZ/DAZ changes subnormal handling only; it
            // does not affect memory safety, and the previous value is
            // restored on drop.
            let saved_mxcsr = unsafe { core::arch::x86_64::_mm_getcsr() };
            unsafe {
                core::arch::x86_64::_mm_setcsr(
                    saved_mxcsr | MXCSR_FLUSH_TO_ZERO | MXCSR_DENORMALS_ARE_ZERO,
                );
            }
            Self {
                saved_mxcsr,
                _not_send_sync: PhantomData,
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            let saved_fpcr: u64;
            // SAFETY: `mrs`/`msr` on FPCR read and write the floating-point
            // control register of the current thread, which is unprivileged
            // architectural state on aarch64. Setting the FZ bit changes
            // subnormal handling only; the previous value is restored on
            // drop. `nostack`/`nomem` hold: the instructions touch neither
            // memory nor the stack.
            unsafe {
                core::arch::asm!(
                    "mrs {saved}, fpcr",
                    "orr {updated}, {saved}, {fz}",
                    "msr fpcr, {updated}",
                    saved = out(reg) saved_fpcr,
                    updated = out(reg) _,
                    fz = in(reg) FPCR_FLUSH_TO_ZERO,
                    options(nostack, nomem, preserves_flags),
                );
            }
            Self {
                saved_fpcr,
                _not_send_sync: PhantomData,
            }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Self {
                _not_send_sync: PhantomData,
            }
        }
    }
}

impl Default for DenormalGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DenormalGuard {
    fn drop(&mut self) {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: restores the exact MXCSR value captured in `new` on the
        // same thread (the guard is `!Send`), returning the control register
        // to its prior state.
        unsafe {
            core::arch::x86_64::_mm_setcsr(self.saved_mxcsr);
        }

        #[cfg(target_arch = "aarch64")]
        // SAFETY: restores the exact FPCR value captured in `new` on the same
        // thread (the guard is `!Send`), returning the control register to
        // its prior state. `nostack`/`nomem` hold as in `new`.
        unsafe {
            core::arch::asm!(
                "msr fpcr, {restored}",
                restored = in(reg) self.saved_fpcr,
                options(nostack, nomem, preserves_flags),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DenormalGuard;
    use std::hint::black_box;

    #[test]
    fn guard_constructs_and_drops_without_crashing() {
        let guard = DenormalGuard::new();
        drop(guard);
        // Nesting restores correctly in LIFO order.
        let outer = DenormalGuard::new();
        let inner = DenormalGuard::new();
        drop(inner);
        drop(outer);
    }

    #[test]
    fn subnormal_multiply_flushes_to_zero_while_guard_is_alive() {
        // A positive subnormal input; halving it stays subnormal without FTZ.
        let subnormal = f32::MIN_POSITIVE / 2.0;

        let guard = DenormalGuard::new();
        let flushed = black_box(black_box(subnormal) * black_box(0.5));
        drop(guard);

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        assert_eq!(
            flushed, 0.0,
            "hardware flush-to-zero should zero subnormal results"
        );

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        assert!(flushed > 0.0, "no-op guard leaves subnormals intact");
    }

    #[test]
    fn fp_state_is_restored_after_drop() {
        {
            let _guard = DenormalGuard::new();
        }
        let subnormal = f32::MIN_POSITIVE / 2.0;
        let result = black_box(black_box(subnormal) * black_box(0.5));
        assert!(
            result > 0.0,
            "subnormal handling should return to default after the guard drops"
        );
    }
}
