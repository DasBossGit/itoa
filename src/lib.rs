//! [![github]](https://github.com/dtolnay/itoa)&ensp;[![crates-io]](https://crates.io/crates/itoa)&ensp;[![docs-rs]](https://docs.rs/itoa)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! <br>
//!
//! This crate provides a fast conversion of integer primitives to decimal
//! strings. The implementation comes straight from [libcore] but avoids the
//! performance penalty of going through [`core::fmt::Formatter`].
//!
//! See also [`zmij`] for printing floating point primitives.
//!
//! [libcore]: https://github.com/rust-lang/rust/blob/1.92.0/library/core/src/fmt/num.rs#L190-L253
//! [`zmij`]: https://github.com/dtolnay/zmij
//!
//! # Example
//!
//! ```
//! fn main() {
//!     let mut buffer = itoa::Buffer::new();
//!     let printed = buffer.format(128u64);
//!     assert_eq!(printed, "128");
//! }
//! ```
//!
//! # Performance
//!
//! The [itoa-benchmark] compares this library and other Rust integer formatting
//! implementations across a range of integer sizes. The vertical axis in this
//! chart shows nanoseconds taken by a single execution of
//! `itoa::Buffer::new().format(value)` so a lower result indicates a faster
//! library.
//!
//! [itoa-benchmark]: https://github.com/dtolnay/itoa-benchmark
//!
//! ![performance](https://raw.githubusercontent.com/dtolnay/itoa/master/itoa-benchmark.png)

#![doc(html_root_url = "https://docs.rs/itoa/1.0.18")]
#![no_std]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::expl_impl_clone_on_copy,
    clippy::identity_op,
    clippy::items_after_statements,
    clippy::must_use_candidate,
    clippy::needless_doctest_main,
    clippy::unreadable_literal,
    incomplete_features
)]
#![feature(const_trait_impl)]

mod u128_ext;

use core::hint;
use core::mem::{self, MaybeUninit};
use core::str;
#[cfg(feature = "no-panic")]
use no_panic::no_panic;

/// A correctly sized stack allocation for the formatted integer to be written
/// into.
///
/// # Example
///
/// ```
/// let mut buffer = itoa::Buffer::new();
/// let printed = buffer.format(1234);
/// assert_eq!(printed, "1234");
/// ```
pub struct Buffer<M: Integer = i128> {
    bytes: M::Buffer,
}

impl<M: Integer> Default for Buffer<M> {
    #[inline]
    fn default() -> Buffer<M> {
        Buffer::new_with()
    }
}

impl<M: Integer> Copy for Buffer<M> {}

#[allow(clippy::non_canonical_clone_impl)]
impl<M: Integer> Clone for Buffer<M> {
    #[inline]
    fn clone(&self) -> Self {
        const { Buffer::new_with() }
    }
}

impl Buffer<i128> {
    /// This is a cheap operation; you don't need to worry about reusing buffers
    /// for efficiency.
    #[inline]
    #[cfg_attr(feature = "no-panic", no_panic)]
    pub fn new() -> Buffer<i128> {
        Buffer {
            bytes: const { [MaybeUninit::<u8>::uninit(); i128::MAX_STR_LEN] },
        }
    }
}

impl<M: Integer> Buffer<M> {
    /// This is a cheap operation; you don't need to worry about reusing buffers
    /// for efficiency.
    #[inline]
    #[cfg_attr(feature = "no-panic", no_panic)]
    pub const fn new_with() -> Buffer<M> {
        Buffer { bytes: M::buffer() }
    }

    /// Print an integer into this buffer and return a reference to its string
    /// representation within the buffer.
    #[cfg_attr(feature = "no-panic", no_panic)]
    pub fn format<I: Integer>(&mut self, i: I) -> &str
    where
        // Ensure that the buffer for M is at least as large as the buffer for I.
        M: SufficientBuffer<I>,
    {
        let buf_ptr = <M as private::Buffer>::as_mut_ptr(&mut self.bytes).cast::<I::Buffer>();
        let string = i.write(unsafe { &mut *buf_ptr });
        if string.len() > I::MAX_STR_LEN {
            unsafe { hint::unreachable_unchecked() };
        }
        string
    }
}

pub const trait SufficientBuffer<Other: Integer>: Integer {}

macro_rules! impl_ge {
    ($A:ty [$($(#[$ptr_w:literal $(| $or_ptr_w:literal)*])?$B:ty)*]) => {
        $(
            $(#[cfg(any(target_pointer_width = $ptr_w $(, target_pointer_width = $or_ptr_w)*))])?
            impl const SufficientBuffer<$B> for $A {}
            $(#[cfg(any(target_pointer_width = $ptr_w $(, target_pointer_width = $or_ptr_w)*))])?
            const _: () = {
                let size_a = size_of::<<$A as private::Buffer>::Buffer>();
                let size_b = size_of::<<$B as private::Buffer>::Buffer>();
                assert!(size_a >= size_b, stringify!($A >= $B));
            };
        )*
    };
}

// i128::MAX_STR_LEN can hold every integer's string representation, thus this blanket implementation is sufficient for all integers.
impl<I: Integer> const SufficientBuffer<I> for i128 {}
impl_ge!(u128 [usize isize u128 u64 i64 u32 i32 u16 i16 u8 i8]);
impl_ge!(i64 [#["64"] usize #["64"] isize u64 i64 u32 i32 u16 i16 u8 i8]);
impl_ge!(u64 [#["64"] usize #["64"] isize u64 i64 u32 i32 u16 i16 u8 i8]);
impl_ge!(i32 [#["32"] usize #["32"] isize u32 i32 u16 i16 u8 i8]);
impl_ge!(u32 [#["32"] usize #["32"] isize u32 u16 i16 u8 i8]);
impl_ge!(i16 [#["16"] usize #["16"] isize u16 i16 u8 i8]);
impl_ge!(u16 [#["16"] usize #["16"] isize u16 u8 i8]);
impl_ge!(i8 [u8 i8]);
impl_ge!(u8[u8]);
impl_ge!(isize [usize isize #["64"] i64 #["64"] u64 #["64" | "32"] i32 #["64" | "32"] u32 #["64" | "32" |"16"] i16 #["64" | "32" |"16"] u16 i8 u8]);
impl_ge!(usize [isize usize #["64"] i64 #["64"] u64 #["64" | "32"] i32 #["64" | "32"] u32 #["64" | "32" |"16"] i16 #["64" | "32" |"16"] u16 i8 u8]);

/// An integer that can be written into an [`itoa::Buffer`][Buffer].
///
/// This trait is sealed and cannot be implemented for types outside of itoa.
pub const trait Integer: private::Sealed {
    /// The maximum length of string that formatting an integer of this type can
    /// produce on the current target platform.
    const MAX_STR_LEN: usize;
}

// Seal to prevent downstream implementations of the Integer trait.
mod private {
    #[doc(hidden)]
    pub trait Sealed: Copy + const Buffer {
        #[doc(hidden)]
        fn write(self, buf: &mut Self::Buffer) -> &str;
    }

    #[doc(hidden)]
    pub const trait Buffer: Copy {
        #[doc(hidden)]
        type Buffer: Copy + 'static;

        #[doc(hidden)]
        fn buffer() -> Self::Buffer;

        #[doc(hidden)]
        fn as_mut_ptr(buf: &mut Self::Buffer) -> *mut super::MaybeUninit<u8>;
    }
}

macro_rules! impl_Integer {
    ($Signed:ident, $Unsigned:ident) => {
        const _: () = {
            assert!($Signed::MIN < 0, "need signed");
            assert!($Unsigned::MIN == 0, "need unsigned");
            assert!($Signed::BITS == $Unsigned::BITS, "need counterparts");
        };

        impl const Integer for $Unsigned {
            const MAX_STR_LEN: usize = $Unsigned::MAX.ilog10() as usize + 1;
        }

        impl private::Sealed for $Unsigned {
            #[inline]
            #[cfg_attr(feature = "no-panic", no_panic)]
            fn write(self, buf: &mut Self::Buffer) -> &str {
                let offset = Unsigned::fmt(self, buf);
                // SAFETY: Starting from `offset`, all elements of the slice have been set.
                unsafe { slice_buffer_to_str(buf, offset) }
            }
        }

        impl const private::Buffer for $Unsigned {
            type Buffer = [MaybeUninit<u8>; Self::MAX_STR_LEN];

            #[inline]
            fn buffer() -> Self::Buffer {
                [MaybeUninit::uninit(); Self::MAX_STR_LEN]
            }

            #[inline]
            fn as_mut_ptr(buf: &mut Self::Buffer) -> *mut MaybeUninit<u8> {
                buf.as_mut_ptr()
            }
        }

        impl const Integer for $Signed {
            const MAX_STR_LEN: usize = $Signed::MAX.ilog10() as usize + 2;
        }

        impl private::Sealed for $Signed {
            #[inline]
            #[cfg_attr(feature = "no-panic", no_panic)]
            fn write(self, buf: &mut Self::Buffer) -> &str {
                let mut offset = Self::MAX_STR_LEN - $Unsigned::MAX_STR_LEN;
                offset += Unsigned::fmt(
                    self.unsigned_abs(),
                    (&mut buf[offset..]).try_into().unwrap(),
                );
                if self < 0 {
                    offset -= 1;
                    buf[offset].write(b'-');
                }
                // SAFETY: Starting from `offset`, all elements of the slice have been set.
                unsafe { slice_buffer_to_str(buf, offset) }
            }
        }

        impl const private::Buffer for $Signed {
            type Buffer = [MaybeUninit<u8>; Self::MAX_STR_LEN];

            #[inline]
            fn buffer() -> Self::Buffer {
                [MaybeUninit::uninit(); Self::MAX_STR_LEN]
            }

            #[inline]
            fn as_mut_ptr(buf: &mut Self::Buffer) -> *mut MaybeUninit<u8> {
                buf.as_mut_ptr()
            }
        }
    };
}

impl_Integer!(i8, u8);
impl_Integer!(i16, u16);
impl_Integer!(i32, u32);
impl_Integer!(i64, u64);
impl_Integer!(i128, u128);

macro_rules! impl_Integer_size {
    ($t:ty as $primitive:ident #[cfg(target_pointer_width = $width:literal)]) => {
        #[cfg(target_pointer_width = $width)]
        impl Integer for $t {
            const MAX_STR_LEN: usize = <$primitive as Integer>::MAX_STR_LEN;
        }

        #[cfg(target_pointer_width = $width)]
        impl private::Sealed for $t {
            #[inline]
            #[cfg_attr(feature = "no-panic", no_panic)]
            fn write(self, buf: &mut Self::Buffer) -> &str {
                (self as $primitive).write(buf)
            }
        }

        #[cfg(target_pointer_width = $width)]
        impl const private::Buffer for $t {
            type Buffer = <$primitive as private::Buffer>::Buffer;

            #[inline]
            fn buffer() -> Self::Buffer {
                <$primitive as private::Buffer>::buffer()
            }

            #[inline]
            fn as_mut_ptr(buf: &mut Self::Buffer) -> *mut MaybeUninit<u8> {
                buf.as_mut_ptr()
            }
        }
    };
}

impl_Integer_size!(isize as i16 #[cfg(target_pointer_width = "16")]);
impl_Integer_size!(usize as u16 #[cfg(target_pointer_width = "16")]);
impl_Integer_size!(isize as i32 #[cfg(target_pointer_width = "32")]);
impl_Integer_size!(usize as u32 #[cfg(target_pointer_width = "32")]);
impl_Integer_size!(isize as i64 #[cfg(target_pointer_width = "64")]);
impl_Integer_size!(usize as u64 #[cfg(target_pointer_width = "64")]);

#[repr(C, align(2))]
struct DecimalPairs([u8; 200]);

// The string of all two-digit numbers in range 00..99 is used as a lookup table.
static DECIMAL_PAIRS: DecimalPairs = DecimalPairs(
    *b"0001020304050607080910111213141516171819\
       2021222324252627282930313233343536373839\
       4041424344454647484950515253545556575859\
       6061626364656667686970717273747576777879\
       8081828384858687888990919293949596979899",
);

// Returns {value / 100, value % 100} correct for values of up to 4 digits.
const fn divmod100(value: u32) -> (u32, u32) {
    debug_assert!(value < 10_000);
    const EXP: u32 = 19; // 19 is faster or equal to 12 even for 3 digits.
    const SIG: u32 = (1 << EXP) / 100 + 1;
    let div = (value * SIG) >> EXP; // value / 100
    (div, value - div * 100)
}

/// This function converts a slice of ascii characters into a `&str` starting
/// from `offset`.
///
/// # Safety
///
/// `buf` content starting from `offset` index MUST BE initialized and MUST BE
/// ascii characters.
#[cfg_attr(feature = "no-panic", no_panic)]
unsafe fn slice_buffer_to_str(buf: &[MaybeUninit<u8>], offset: usize) -> &str {
    // SAFETY: `offset` is always included between 0 and `buf`'s length.
    let written = unsafe { buf.get_unchecked(offset..) };
    // SAFETY: (`assume_init_ref`) All buf content since offset is set.
    // SAFETY: (`from_utf8_unchecked`) Writes use ASCII from the lookup table exclusively.
    unsafe { str::from_utf8_unchecked(&*(written as *const [MaybeUninit<u8>] as *const [u8])) }
}

const trait Unsigned: Integer {
    fn fmt(self, buf: &mut Self::Buffer) -> usize;
}

macro_rules! impl_Unsigned {
    ($Unsigned:ident) => {
        impl Unsigned for $Unsigned {
            #[cfg_attr(feature = "no-panic", no_panic)]
            fn fmt(self, buf: &mut Self::Buffer) -> usize {
                // Count the number of bytes in buf that are not initialized.
                let mut offset = buf.len();
                // Consume the least-significant decimals from a working copy.
                let mut remain = self;

                // Format per four digits from the lookup table.
                // Four digits need a 16-bit $Unsigned or wider.
                while mem::size_of::<Self>() > 1
                    && remain
                        > 999
                            .try_into()
                            .expect("branch is not hit for types that cannot fit 999 (u8)")
                {
                    offset -= 4;

                    // pull two pairs
                    let scale: Self = 1_00_00
                        .try_into()
                        .expect("branch is not hit for types that cannot fit 1E4 (u8)");
                    let quad = remain % scale;
                    remain /= scale;
                    let (pair1, pair2) = divmod100(quad as u32);
                    unsafe {
                        buf[offset + 0]
                            .write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0));
                        buf[offset + 1]
                            .write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1));
                        buf[offset + 2]
                            .write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0));
                        buf[offset + 3]
                            .write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1));
                    }
                }

                // Format per two digits from the lookup table.
                if remain > 9 {
                    offset -= 2;

                    let (last, pair) = divmod100(remain as u32);
                    remain = last as Self;
                    unsafe {
                        buf[offset + 0]
                            .write(*DECIMAL_PAIRS.0.get_unchecked(pair as usize * 2 + 0));
                        buf[offset + 1]
                            .write(*DECIMAL_PAIRS.0.get_unchecked(pair as usize * 2 + 1));
                    }
                }

                // Format the last remaining digit, if any.
                if remain != 0 || self == 0 {
                    offset -= 1;

                    // Either the compiler sees that remain < 10, or it prevents
                    // a boundary check up next.
                    let last = remain as u8 & 15;
                    buf[offset].write(b'0' + last);
                    // not used: remain = 0;
                }

                offset
            }
        }
    };
}

impl_Unsigned!(u8);
impl_Unsigned!(u16);
impl_Unsigned!(u32);
impl_Unsigned!(u64);

impl Unsigned for u128 {
    #[cfg_attr(feature = "no-panic", no_panic)]
    fn fmt(self, buf: &mut Self::Buffer) -> usize {
        // Optimize common-case zero, which would also need special treatment due to
        // its "leading" zero.
        if self == 0 {
            let offset = buf.len() - 1;
            buf[offset].write(b'0');
            return offset;
        }
        // Take the 16 least-significant decimals.
        let (quot_1e16, mod_1e16) = div_rem_1e16(self);
        let (mut remain, mut offset) = if quot_1e16 == 0 {
            (mod_1e16, u128::MAX_STR_LEN)
        } else {
            // Write digits at buf[23..39].
            enc_16lsd::<{ u128::MAX_STR_LEN - 16 }>(buf, mod_1e16);

            // Take another 16 decimals.
            let (quot2, mod2) = div_rem_1e16(quot_1e16);
            if quot2 == 0 {
                (mod2, u128::MAX_STR_LEN - 16)
            } else {
                // Write digits at buf[7..23].
                enc_16lsd::<{ u128::MAX_STR_LEN - 32 }>(buf, mod2);
                // Quot2 has at most 7 decimals remaining after two 1e16 divisions.
                (quot2 as u64, u128::MAX_STR_LEN - 32)
            }
        };

        // Format per four digits from the lookup table.
        while remain > 999 {
            offset -= 4;

            // pull two pairs
            let quad = remain % 1_00_00;
            remain /= 1_00_00;
            let (pair1, pair2) = divmod100(quad as u32);
            unsafe {
                buf[offset + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0));
                buf[offset + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1));
                buf[offset + 2].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0));
                buf[offset + 3].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1));
            }
        }

        // Format per two digits from the lookup table.
        if remain > 9 {
            offset -= 2;

            let (last, pair) = divmod100(remain as u32);
            remain = last as u64;
            unsafe {
                buf[offset + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair as usize * 2 + 0));
                buf[offset + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair as usize * 2 + 1));
            }
        }

        // Format the last remaining digit, if any.
        if remain != 0 {
            offset -= 1;

            // Either the compiler sees that remain < 10, or it prevents
            // a boundary check up next.
            let last = remain as u8 & 15;
            buf[offset].write(b'0' + last);
            // not used: remain = 0;
        }
        offset
    }
}

// Encodes the 16 least-significant decimals of n into `buf[OFFSET..OFFSET + 16]`.
#[cfg_attr(feature = "no-panic", no_panic)]
fn enc_16lsd<const OFFSET: usize>(buf: &mut [MaybeUninit<u8>], n: u64) {
    // Consume the least-significant decimals from a working copy.
    let mut remain = n;

    // Format per four digits from the lookup table.
    for quad_index in (1..4).rev() {
        // pull two pairs
        let quad = remain % 1_00_00;
        remain /= 1_00_00;
        let (pair1, pair2) = divmod100(quad as u32);
        unsafe {
            buf[quad_index * 4 + OFFSET + 0]
                .write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0));
            buf[quad_index * 4 + OFFSET + 1]
                .write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1));
            buf[quad_index * 4 + OFFSET + 2]
                .write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0));
            buf[quad_index * 4 + OFFSET + 3]
                .write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1));
        }
    }

    // final two pairs
    let (pair1, pair2) = divmod100(remain as u32);
    unsafe {
        buf[OFFSET + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0));
        buf[OFFSET + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1));
        buf[OFFSET + 2].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0));
        buf[OFFSET + 3].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1));
    }
}

// Euclidean division plus remainder with constant 1E16 basically consumes 16
// decimals from n.
//
// The integer division algorithm is based on the following paper:
//
//   T. Granlund and P. Montgomery, “Division by Invariant Integers Using Multiplication”
//   in Proc. of the SIGPLAN94 Conference on Programming Language Design and
//   Implementation, 1994, pp. 61–72
//
#[cfg_attr(feature = "no-panic", no_panic)]
fn div_rem_1e16(n: u128) -> (u128, u64) {
    const D: u128 = 1_0000_0000_0000_0000;
    // The check inlines well with the caller flow.
    if n < D {
        return (0, n as u64);
    }

    // These constant values are computed with the CHOOSE_MULTIPLIER procedure
    // from the Granlund & Montgomery paper, using N=128, prec=128 and d=1E16.
    const M_HIGH: u128 = 76624777043294442917917351357515459181;
    const SH_POST: u8 = 51;

    // n.widening_mul(M_HIGH).1 >> SH_POST
    let quot = u128_ext::mulhi(n, M_HIGH) >> SH_POST;
    let rem = n - quot * D;
    (quot, rem as u64)
}
