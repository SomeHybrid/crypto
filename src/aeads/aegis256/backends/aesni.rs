use core::arch::x86_64::*;

#[derive(Clone, Copy)]
pub struct Block(__m128i);

impl Block {
    #[inline(always)]
    pub fn load(items: &[u8]) -> Block {
        Block(unsafe { _mm_loadu_si128(items.as_ptr() as *const __m128i) })
    }

    #[inline(always)]
    pub fn store(&self) -> [u8; 16] {
        let mut output = [0u8; 16];
        unsafe { _mm_storeu_si128(output.as_mut_ptr() as *mut _, self.0) };
        output
    }

    #[inline(always)]
    pub fn xor(&self, other: Block) -> Block {
        Block(unsafe { _mm_xor_si128(self.0, other.0) })
    }

    #[inline(always)]
    pub fn enc(&self, other: Block) -> Block {
        Block(unsafe { _mm_aesenc_si128(self.0, other.0) })
    }

    #[inline(always)]
    pub fn and(&self, other: Block) -> Block {
        Block(unsafe { _mm_and_si128(self.0, other.0) })
    }
}
