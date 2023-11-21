use cfg_if::cfg_if;

mod fallback;

cfg_if! {
    if #[cfg(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        )
    )] {
        mod avx2;
        mod sse2;
        pub use avx2::Backend;
        pub use sse2::hchacha;
    } else if #[cfg(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2"
        )
    )] {
        mod sse2;
        pub use sse2::Backend;
        pub use sse2::hchacha;
    }
    else {
        pub use fallback::Backend;
        pub use fallback::hchacha;
    }
}
