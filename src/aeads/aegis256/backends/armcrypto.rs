use core::arch::aarch64::*;
use core::ops::{BitAnd, BitXor};

#[derive(Clone, Copy)]
pub struct Block(uint8x16_t);

impl Block {
    #[inline(always)]
    pub fn load(items: &[u8]) -> Block {
        Block(unsafe { vld1q_u8(items.as_ptr() as *const _) })
    }

    #[inline(always)]
    pub fn store(&self) -> [u8; 16] {
        let mut output = [0u8; 16];
        unsafe { vst1q_u8(output.as_mut_ptr() as *mut _, self.0) };
        output
    }

    #[inline(always)]
    pub fn enc(&self, other: Block) -> Block {
        Block(unsafe { vaeseq_u8(self.0, other.0) })
    }
}

impl BitAnd for Block {
    type Output = Block;

    #[inline(always)]
    fn bitand(self, other: Self) -> Self::Output {
        Block(unsafe { vandq_u8(self.0, other.0) })
    }
}

impl BitXor for Block {
    type Output = Block;

    #[inline(always)]
    fn bitxor(self, other: Self) -> Self::Output {
        Block(unsafe { veorq_u8(self.0, other.0) })
    }
}

impl BitAnd for &Block {
    type Output = Block;

    #[inline(always)]
    fn bitand(self, other: Self) -> Self::Output {
        *self & *other
    }
}

impl BitXor for &Block {
    type Output = Block;

    #[inline(always)]
    fn bitxor(self, other: Self) -> Self::Output {
        *self ^ *other
    }
}

impl BitXor<&Block> for Block {
    type Output = Block;

    #[inline(always)]
    fn bitxor(self, other: &Block) -> Self::Output {
        &self ^ other
    }
}
