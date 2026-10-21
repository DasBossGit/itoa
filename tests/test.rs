#![allow(non_snake_case)]

macro_rules! test {
    ($($name:ident($value:expr, $expected:expr, $ty:ty))*) => {
        $(
            #[test]
            fn $name() {
                let mut buffer = itoa::Buffer::<$ty>::new_with();
                let s = buffer.format($value);
                assert_eq!(s, $expected);
            }
        )*
    }
}

mod default {
    test! {
        test_u64_0(0u64, "0", i128)
        test_u64_half(u64::from(u32::MAX), "4294967295", i128)
        test_u64_max(u64::MAX, "18446744073709551615", i128)
        test_i64_min(i64::MIN, "-9223372036854775808", i128)

        test_i16_0(0i16, "0", i128)
        test_i16_min(i16::MIN, "-32768", i128)

        test_u128_0(0u128, "0", i128)
        test_u128_max(u128::MAX, "340282366920938463463374607431768211455", i128)
        test_i128_min(i128::MIN, "-170141183460469231731687303715884105728", i128)
        test_i128_max(i128::MAX, "170141183460469231731687303715884105727", i128)
    }
}

mod typed {
    test! {
        test_u64_0(0u64, "0", u64)
        test_u64_half(u64::from(u32::MAX), "4294967295", u64)
        test_u64_max(u64::MAX, "18446744073709551615", u64)
        test_i64_min(i64::MIN, "-9223372036854775808", i64)

        test_i16_0(0i16, "0", i16)
        test_i16_min(i16::MIN, "-32768", i16)

        test_u128_0(0u128, "0", u128)
        test_u128_max(u128::MAX, "340282366920938463463374607431768211455", u128)
        test_i128_min(i128::MIN, "-170141183460469231731687303715884105728", i128)
        test_i128_max(i128::MAX, "170141183460469231731687303715884105727", i128)
    }
}

#[test]
fn test_max_str_len() {
    use itoa::Integer as _;

    assert_eq!(i8::MAX_STR_LEN, 4);
    assert_eq!(u8::MAX_STR_LEN, 3);
    assert_eq!(i16::MAX_STR_LEN, 6);
    assert_eq!(u16::MAX_STR_LEN, 5);
    assert_eq!(i32::MAX_STR_LEN, 11);
    assert_eq!(u32::MAX_STR_LEN, 10);
    assert_eq!(i64::MAX_STR_LEN, 20);
    assert_eq!(u64::MAX_STR_LEN, 20);
    assert_eq!(i128::MAX_STR_LEN, 40);
    assert_eq!(u128::MAX_STR_LEN, 39);
}

fn test_generic(value: impl itoa::Integer, expected: &str) {
    let mut buffer = itoa::Buffer::<u64>::new_with();
    let s = buffer.format(value);
    assert_eq!(s, expected);
}
