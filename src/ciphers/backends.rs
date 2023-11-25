use cfg_if::cfg_if;
pub mod fallback;

cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        pub mod avx2;
        pub mod sse2;
    }
}
