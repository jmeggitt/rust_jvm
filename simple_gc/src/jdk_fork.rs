use bitflags::bitflags;
use std::sync::atomic::{AtomicU32, Ordering};

#[repr(transparent)]
pub struct MarkHeader(AtomicU32);

// #[cfg(target_pointer_width = "32")]
bitflags! {
    pub struct MarkSections: u32 {
        // Normal Object
        const HASH        = 0xFFFF_FF80;
        const AGE         = 0x0000_0078;
        const BIASED_LOCK = 0x0000_0004;
        const LOCK        = 0x0000_0003;

        // Biased Object
        const THREAD      = 0xFFFF_FE00;
        const EPOCH       = 0x0000_0180;

        // CMS Free Block
        const SIZE        = 0xFFFF_FFFF;

        // CMS Promoted Object
        const PROMOTED    = 0xFFFF_FFF8;
        const PROMO_BITS  = 0x0000_0007;
    }
}
//    [JavaThread* | epoch | age | 1 | 01]       lock is biased toward given thread
//    [0           | epoch | age | 1 | 01]       lock is anonymously biased
//
//  - the two lock bits are used to describe three states: locked/unlocked and monitor.
//
//    [ptr             | 00]  locked             ptr points to real header on stack
//    [header      | 0 | 01]  unlocked           regular object header
//    [ptr             | 10]  monitor            inflated lock (header is wapped out)
//    [ptr             | 11]  marked             used by markSweep to mark an object
//                                               not valid at any other time
//
//    We assume that stack/thread pointers have the lowest two bits cleared.

impl MarkHeader {
    const LOCKED_VALUE: u32 = 0;
    const UNLOCKED_VALUE: u32 = 1;
    const MONITOR_VALUE: u32 = 2;
    const MARKED_VALUE: u32 = 3;
    const BIASED_LOCK_PATTERN: u32 = 5;

    pub fn value(&self) -> u32 {
        self.0.load(Ordering::SeqCst)
    }

    // pub fn has_bias_pattern(&self) -> bool {
    //     self.value() & MarkSections::
    // }
}

// #[cfg(target_pointer_width = "64")]
// bitflags! {
//     pub struct MarkSections: u64 {
//         // Normal Object
//         const LOCK = 0x0000_0000_0000_0003;
//         const BIASED_LOCK = 0x0000_0000_0000_0004;
//         const AGE = 0x0000_0000_0000_0078;
//         // const _UNUSED = 0x0000_0000_0000_0080;
//         const HASH = 0x0000_007F_FFFF_FF00;
//         // const _UNUSED = 0xFFFF_FF80_0000_0000;
//
//         // Biased Object
//         const EPOCH = 0x0000_0000_0000_0300;
//         const THREAD = 0xFFFF_FFFF_FFFF_FC00;
//
//         // CMS Promoted Object
//         const PROMOTED_OBJECT = 0xFFFF_FFFF_FFFF_FFF8;
//         const PROMOTION_BITS = 0x0000_0000_0000_0007;
//     }
// }
