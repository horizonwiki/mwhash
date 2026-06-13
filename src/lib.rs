//! src/lib.rs
//! mwhash - a simple, fast, non-cryptographic hash function for Rust
//! 
//! ## Usage Examples
//!
//! ### 1. One-shot hashing
//! Use this when you have all your data ready and just need a quick hash.
//! ```rust
//! use mwhash::mwhash;
//! 
//! let hash = mwhash(b"hello mwhash");
//! println!("Result: {:08x}", hash);
//! 
//! assert_ne!(hash, 0); 
//! ```
//!
//! ### 2. Incremental hashing
//! Use this when your data arrives in chunks (like reading from a file or network). 
//! It saves memory because you don't need to keep the whole input in RAM.
//! ```rust
//! use mwhash::{Hasher, mwhash};
//!
//! let mut h = Hasher::new();
//! h.update(b"hello");
//! h.update(b" rust"); // println!("hash: {:08x}", h.finish());
//! 
//! // Must call .finish() to finalize the calculation
//! let hash = h.finish();
//! println!("Incremental hash: {:08x}", hash);
//! 
//! assert_eq!(hash, mwhash(b"hello rust"));
//! ```
//!
//! ### 3. Hashing with a custom seed
//! Use this to create unique hashes for different contexts or to "salt" your data 
//! so identical inputs produce different outputs.
//! ```rust
//! use mwhash::{mwhash, Hasher};
//!
//! let mut h = Hasher::with_seed(0x12345678);
//! h.update(b"codeberg better than github");
//! let h1 = h.finish();
//! 
//! println!("Seeded hash: {:08x}", h1);
//! // The hash with a seed will be different from the default one
//! assert_ne!(h1, mwhash(b"codeberg better than github"));
//! ```
//! 
//! ### 4. Hashing with a custom string seed
//! Use this for a more human-friendly approach. The string is automatically 
//! hashed to become the numeric seed.
//! ```rust
//! use mwhash::{mwhash, Hasher};
//!
//! let mut h = Hasher::with_string_seed("My-SID-is-definitely-unique");
//! h.update(b"100% essential data");
//! let hash = h.finish();
//! println!("String-seeded hash: {:08x}", hash);
//! 
//! assert_ne!(hash, 0);
//! assert_ne!(hash, mwhash(b"100% essential data"));
//! ```
//! 
//! ### 5. Your SID is definitely better than 0x9E3779B9?
//! Because typing `0x9E3779B9` is boring, and "My-SID-is-definitely-unique" 
//! is a statement of intent!
//! 
//! ### 6. Concatenation helper
//! If you are processing data in parts, you can easily combine them.
//! This shows that the order of updates doesn't change the final result.
//! ```rust
//! use mwhash::{mwhash, mwhash_concat};
//!
//! let part1 = b"foo";
//! let part2 = b"bar";
//! 
//! // Both approaches yield the exact same hash
//! let combined_hash = mwhash_concat(part1, part2);
//! let full_hash = mwhash(b"foobar");
//!
//! println!("Hash: {:08x}", combined_hash);
//! assert_eq!(combined_hash, full_hash);
//! ```
//! ### 7. One-shot hashing with a custom seed
//! Use this when you have data and a seed, but don't need to process 
//! data in multiple chunks. It's the fastest way to get a seeded hash.
//! ```rust
//! use mwhash::mwhash_seeded;
//!
//! let data = b"test";
//! let seed = 0xDEAD_BEEF;
//! 
//! let hash = mwhash_seeded(data, seed);
//! println!("Hash: {:08x}", hash);
//! 
//! // Verify that it's not the default hash
//! assert_ne!(hash, mwhash::mwhash(data));
//! ```
//! ### 8. Reusing the Hasher
//! You can reuse the same Hasher instance by calling `.reset()`. 
//! This is more efficient than creating a new instance for every hash operation
//! because it reuses existing memory.
//! ```rust
//! use mwhash::{mwhash_seeded, Hasher};
//!
//! let custom_seed = 0x12345678;
//! let mut h = Hasher::with_seed(custom_seed);
//!
//! // First usage
//! h.update(b"data1");
//! let hash1 = h.finish();
//! println!("Hash 1: {:08x}", hash1);
//!
//! // Reset and reuse
//! h.reset();
//! h.update(b"data2");
//! let hash2 = h.finish();
//! println!("Hash 2: {:08x}", hash2);
//!
//! assert_eq!(hash2, mwhash_seeded(b"data2", custom_seed));
//! assert_ne!(hash1, hash2);
//! ```
//! ### 9. Resetting the Hasher
//! You can reset a Hasher to reuse it without reallocating memory.
//! After a reset, the Hasher behaves exactly as it did when it was first created
//! with the same seed.
//! ```rust
//! use mwhash::Hasher;
//!
//! let custom_seed = 0x12345678;
//! let mut h = Hasher::with_seed(custom_seed);
//!
//! // 1. Use the hasher
//! h.update(b"some data");
//! println!("Hash: {:08x}", h.finish());
//! 
//! // 2. Reset the hasher
//! h.reset();
//! 
//! // 3. It's now empty again, behaving like a fresh instance
//! // An empty hasher with a seed should equal mwhash_seeded(b"", seed)
//! assert_eq!(h.finish(), mwhash::mwhash_seeded(b"", custom_seed));
//! 
//! // 4. Reuse it for new data
//! h.update(b"new data");
//! println!("New hash after reset: {:08x}", h.finish());
//! ```

#![no_std]

const SEED_DEFAULT: u32 = 0x9E37_79B9;
const MUL_A: u32 = 0x85EB_CA6B;
const MUL_B: u32 = 0xC2B2_AE35;
const FINAL_XOR_1: u32 = 0x6B43_A9B5;
const FINAL_XOR_2: u32 = 0x52DC_E729;

#[inline(always)]
fn mix_word(acc: u32, word: u32) -> u32 {
    let mut a = acc.wrapping_add(word.wrapping_mul(MUL_A));
    a = a.rotate_left(13);
    a ^ (a >> 7)
}

#[inline(always)]
fn mix_byte(acc: u32, byte: u8) -> u32 {
    let v = (byte as u32).wrapping_mul(MUL_A);
    let acc = acc.wrapping_add(v);
    let acc = acc.rotate_left(13);
    acc ^ (acc >> 7)
}

#[inline(always)]
fn finalize(mut h: u32, len: u32) -> u32 {
    h = h.wrapping_add(len.wrapping_mul(MUL_A));
    h ^= h >> 16;
    h = h.wrapping_mul(FINAL_XOR_1);
    h ^= h >> 13;
    h = h.wrapping_mul(MUL_B);
    h ^= h >> 16;
    h ^ FINAL_XOR_2
}

#[inline]
pub fn mwhash(data: &[u8]) -> u32 {
    Hasher::new().feed(data).finish()
}

#[inline]
pub fn mwhash_seeded(data: &[u8], seed: u32) -> u32 {
    Hasher::with_seed(seed).feed(data).finish()
}

#[derive(Clone, Debug)]
pub struct Hasher {
    acc: u32,
    seed: u32,
    len: u32,
    tail: [u8; 4],
    tail_len: usize,
}

impl Hasher {
    #[inline]
    pub const fn new() -> Self {
        Self { acc: SEED_DEFAULT, seed: SEED_DEFAULT, len: 0, tail: [0; 4], tail_len: 0 }
    }

    #[inline]
    pub const fn with_seed(seed: u32) -> Self {
        Self { acc: seed, seed, len: 0, tail: [0; 4], tail_len: 0 }
    }

    pub fn update(&mut self, mut data: &[u8]) {
        self.len = self.len.wrapping_add(data.len() as u32);

        if self.tail_len > 0 {
            let take = core::cmp::min(4 - self.tail_len, data.len());
            self.tail[self.tail_len..self.tail_len + take].copy_from_slice(&data[..take]);
            self.tail_len += take;
            data = &data[take..];
            if self.tail_len == 4 {
                let word = u32::from_le_bytes(self.tail);
                self.acc = mix_word(self.acc, word);
                self.tail_len = 0;
            }
        }

        let mut chunks = data.chunks_exact(4);
        for chunk in &mut chunks {
            let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            self.acc = mix_word(self.acc, word);
        }

        let rem = chunks.remainder();
        if !rem.is_empty() {
            self.tail[..rem.len()].copy_from_slice(rem);
            self.tail_len = rem.len();
        }
    }

    #[inline]
    pub fn feed(&mut self, data: &[u8]) -> &mut Self {
        self.update(data);
        self
    }

    #[inline]
    pub fn finish(&self) -> u32 {
        let mut current_acc = self.acc;
        for i in 0..self.tail_len {
            current_acc = mix_byte(current_acc, self.tail[i]);
        }
        finalize(current_acc, self.len)
    }

    #[inline]
    pub fn reset(&mut self) {
        self.acc = self.seed;
        self.len = 0;
        self.tail_len = 0;
    }

    #[inline]
    pub fn reset_with_seed(&mut self, seed: u32) {
        self.seed = seed;
        self.acc = seed;
        self.len = 0;
        self.tail_len = 0;
    }

    pub fn with_string_seed(seed: &str) -> Self {
        let seed = mwhash(seed.as_bytes());
        Self::with_seed(seed)
    }

    pub fn reset_with_string_seed(&mut self, seed: &str) {
        let seed = mwhash(seed.as_bytes());
        self.reset_with_seed(seed);
    }
}

impl Default for Hasher {
    fn default() -> Self { Self::new() }
}

#[inline]
pub fn mwhash_u32(value: u32) -> u32 { mwhash(&value.to_le_bytes()) }
#[inline]
pub fn mwhash_u64(value: u64) -> u32 { mwhash(&value.to_le_bytes()) }
#[inline]
pub fn mwhash_concat(a: &[u8], b: &[u8]) -> u32 {
    let mut h = Hasher::new();
    h.update(a);
    h.update(b);
    h.finish()
}