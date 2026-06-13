// tests/mwhash_tests.rs

use mwhash::*;

    // Verify that the result is the same regardless of how the data is divided into segments
    #[test]
    fn split_invariant_exhaustive() {
        let data = b"abcdefghij"; // 10 bytes is not a multiple of 4
        let expected = mwhash(data);
        for split in 1..data.len() {
            let mut h = Hasher::new();
            h.update(&data[..split]);
            h.update(&data[split..]);
            assert_eq!(
                h.finish(), expected,
                "split at {split} produces a different result"
            );
        }
    }

    // Verify that resetting the hasher does not cause contamination using `tail`.
    #[test]
    fn reset_tail_clean() {
        let mut h = Hasher::new();
        h.update(b"abc"); 
        h.reset();
        assert_eq!(h.finish(), Hasher::new().finish());

        h.update(b"x");
        assert_eq!(h.finish(), mwhash(b"x"));
    }

    // The `len` variable overflows silently—let's check that the wrapping doesn't break determinism.
    #[test]
    fn len_wrapping_is_deterministic() {
        let chunk = &[0xABu8; 256];
        let mut h1 = Hasher::new();
        let mut h2 = Hasher::new();
 
        for _ in 0..100 {
            h1.update(chunk);
            h2.update(chunk);
        }
        assert_eq!(h1.finish(), h2.finish());
    }

    // Tail_len > 4 impossible, but let's check edge cases of update
    #[test]
    fn update_empty_slice_is_noop() {
        let mut h = Hasher::new();
        h.update(b"hello");
        let before = h.finish();
        h.update(b""); // an empty hash — should not change the hash
        assert_eq!(h.finish(), before);
    }

    // Confirms that identical inputs always yield identical hash outputs.
    #[test]
    fn deterministic() {
        assert_eq!(mwhash(b"hello"), mwhash(b"hello"));
        assert_eq!(mwhash(b""), mwhash(b""));
    }

    // Verifies that resetting a seeded hasher reverts it to the original custom seed,
    // not the default seed.
    #[test]
    fn reset_preserves_custom_seed() {
        let custom_seed = 0xDEAD_BEEF;
        let mut h = Hasher::with_seed(custom_seed);
        h.update(b"temporary data");
        h.reset();
        assert_eq!(h.finish(), mwhash_seeded(b"", custom_seed));
        assert_ne!(h.finish(), mwhash(b""));
    }

    // Confirms that even minor changes in input (like capitalization or character 
    // substitution) result in completely different hash values.
    #[test]
    fn different_inputs_different_hashes() {
        assert_ne!(mwhash(b"hello"), mwhash(b"world"));
        assert_ne!(mwhash(b"hello"), mwhash(b"Hello"));
        assert_ne!(mwhash(b"abc"), mwhash(b"abd"));
    }

    // Ensures that an empty input doesn't return a zero-hash, which helps avoid 
    // potential edge-case collisions.
    #[test]
    fn empty_not_zero() { assert_ne!(mwhash(b""), 0); }

    // Verifies that the length of the input is factored into the hash, 
    // ensuring that "a" and "aa" produce distinct outputs.
    #[test]
    fn length_matters() {
        assert_ne!(mwhash(b"a"), mwhash(b"aa"));
        assert_ne!(mwhash(b"aaa"), mwhash(b"aaaa"));
    }

    // Confirms that hashing data piece-by-piece produces the exact same result 
    // as hashing the entire sequence in one go.
    #[test]
    fn incremental_equals_one_shot() {
        let expected = mwhash(b"helloworld");
        let mut h = Hasher::new();
        h.update(b"hello");
        h.update(b"world");
        assert_eq!(h.finish(), expected);
    }

    // Validates the helper function to ensure that concatenating two buffers 
    // and hashing them is equivalent to hashing the joined buffer.
    #[test]
    fn concat_helper() {
        assert_eq!(mwhash_concat(b"foo", b"bar"), mwhash(b"foobar"));
    }

    // Ensures that providing a custom seed actually produces a unique hash 
    // compared to the default seed.
    #[test]
    fn seeded_differs_from_default() {
        assert_ne!(mwhash(b"test"), mwhash_seeded(b"test", 0xDEAD_BEEF));
    }

    // Verifies the utility function that allows hashing primitive u32 values directly.
    #[test]
    fn u32_helper() {
        assert_eq!(mwhash_u32(42), mwhash(&42u32.to_le_bytes()));
    }

    // Confirms that the fluent API allows chaining multiple feed() calls 
    // cleanly without side effects.
    #[test]
    fn feed_chain() {
        assert_eq!(Hasher::new().feed(b"foo").feed(b"bar").finish(), mwhash(b"foobar"));
    }

    // Verifies that the reset method fully restores the hasher to its pristine, 
    // initial state.
    #[test]
    fn reset() {
        let mut h = Hasher::new();
        h.update(b"some data");
        h.reset();
        assert_eq!(h.finish(), Hasher::new().finish());
    }

    // Tests the "avalanche effect": changing a single bit in the input should flip 
    // a significant portion of bits in the resulting hash.
    #[test]
    fn avalanche_single_bit() {
        let a = mwhash(b"\x00\x00\x00\x00");
        let b = mwhash(b"\x01\x00\x00\x00");
        let diff = (a ^ b).count_ones();
        assert!(diff >= 8, "bad avalanche: only {} bits have changed", diff);
    }

    // Ensures that sequences of all zeros and all ones generate distinct, 
    // non-colliding hash values.
    #[test]
    fn all_zeros_vs_all_ones() {
        assert_ne!(mwhash(&[0u8; 32]), mwhash(&[0xFFu8; 32]));
    }

    // Verifies that the hasher can act as a "checkpoint"—you can get a hash 
    // for a prefix, continue adding more data, and get a new hash for the full length.
    #[test]
    fn intermediate_checkpoint() {
        let mut h = Hasher::new();
        h.update(b"prefix");
        let mid = h.finish();
        h.update(b"suffix");
        let full = h.finish();
        assert_ne!(mid, full);
        assert_eq!(mid, mwhash(b"prefix"));
    }
