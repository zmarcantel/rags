#[cfg(test)]
mod count {
    use crate::*;

    // TODO: short_count(...) and long_count(...) tests

    fn generic_count<T>(initial: T, step: T, expect: T)
        where T: std::ops::AddAssign + Clone + std::fmt::Display + std::cmp::PartialEq
    {
        let mut count: T = initial;

        Parser::from_strings(string_vec!("argv[0]", "-v", "-v", "--verbose", "-v", "--verbose"))
            .count('v', "verbose", "increase verbosity", &mut count, step).expect("bad count parse")
        ;

        assert!(count == expect, "unexpected count value {}, wanted {}", count, expect);
    }

    #[test] fn u8() { generic_count::<u8>(0u8, 1u8, 5u8); }
    #[test] fn u8_stepped() { generic_count::<u8>(0u8, 10u8, 50); }

    #[test] fn u16() { generic_count::<u16>(0u16, 1u16, 5u16); }
    #[test] fn u16_stepped() { generic_count::<u16>(0u16, 8u16, 40u16); }

    #[test] fn u32() { generic_count::<u32>(0u32, 1u32, 5u32); }
    #[test] fn u32_stepped() { generic_count::<u32>(0u32, 2u32, 10u32); }

    #[test] fn u64() { generic_count::<u64>(0u64, 1u64, 5u64); }
    #[test] fn u64_stepped() { generic_count::<u64>(0u64, 3u64, 15u64); }

    #[test] fn usize() { generic_count::<usize>(0usize, 1usize, 5usize); }
    #[test] fn usize_stepped() { generic_count::<usize>(0usize, 6usize, 30usize); }


    #[test] fn i8() { generic_count::<i8>(0i8, 1i8, 5i8); }
    #[test] fn i8_stepped() { generic_count::<i8>(0i8, 4i8, 20i8); }

    #[test] fn i16() { generic_count::<i16>(0i16, 1i16, 5i16); }
    #[test] fn i16_stepped() { generic_count::<i16>(0i16, 8i16, 40i16); }

    #[test] fn i32() { generic_count::<i32>(0i32, 1i32, 5i32); }
    #[test] fn i32_stepped() { generic_count::<i32>(0i32, 2i32, 10i32); }

    #[test] fn i64() { generic_count::<i64>(0i64, 1i64, 5i64); }
    #[test] fn i64_stepped() { generic_count::<i64>(0i64, 9i64, 45i64); }

    #[test] fn isize() { generic_count::<isize>(0isize, 1isize, 5isize); }
    #[test] fn isize_stepped() { generic_count::<isize>(0isize, 10isize, 50isize); }


    #[test] fn f32() { generic_count::<f32>(0f32, 1f32, 5f32); }
    #[test] fn f32_stepped() { generic_count::<f32>(0f32, 1.5f32, 7.5f32); }

    #[test] fn f64() { generic_count::<f64>(0f64, 1f64, 5f64); }
    #[test] fn f64_stepped() { generic_count::<f64>(0f64, 3.2f64, 16f64); }


    #[test]
    fn runs() {
        let expect: usize = 10;
        let mut count: usize = 0;
        Parser::from_strings(string_vec!("argv[0]", "-vvvvv", "-vvvvv"))
            .count('v', "verbose", "increase verbosity", &mut count, 1)
                .expect("bad count parse")
        ;

        assert!(count == expect, "unexpected count value {}, wanted {}", count, expect);
    }

    #[test]
    fn runs_interleaved() {
        let expect: usize = 10;
        let mut count: usize = 0;
        let mut other: usize = 0;
        Parser::from_strings(string_vec!("argv[0]", "-vxvxvxvxvx", "-vxvxvxvxvx"))
            .count('v', "verbose", "increase verbosity", &mut count, 1)
                .expect("bad count parse")
            .count('x', "extreme", "do something different", &mut other, 1)
                .expect("bad count parse")
        ;

        assert!(count == expect, "unexpected count value {}, wanted {}", count, expect);
        assert!(other == expect, "unexpected other value {}, wanted {}", other, expect);
    }
}

