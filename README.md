# mwhash

A fast, non-cryptographic `u32` hash. `no_std`, zero dependencies, zero allocations.

Useful for: integrity checks on packets, checksums, fast buffer comparison, hash tables, embedded projects.
Not suitable for cryptography.

---

## Performance

Benchmark results (seed = `0x12345678`) are shown below:

```text
----------------------------------------------------------------------------------------------------
64 KB      | size:     64.00 KB  | iters:  20000 | total: 339.070ms | per call:   16.953µs | throughput:   3.87 GB/s
1 MB       | size:   1024.00 KB  | iters:   2000 | total: 549.020ms | per call:  274.509µs | throughput:   3.82 GB/s
1 GB       | size: 1048576.00 KB | iters:      3 | total: 979.626ms | per call:  326.542ms | throughput:   3.29 GB/s
----------------------------------------------------------------------------------------------------
```

* **64 KB:** L1/L2 cache.
* **1 MB:** L3 cache.
* **1 GB:** main memory.

> **Environment:*** Tested on AMD64 architecture (Processor: [Intel Core Ultra 5 125h], RAM: [16 GB lpddr5x 6000MT/s]).
> **Source code:** [examples/benchmark.rs](./examples/benchmark.rs)

---

## Installation

```toml
[dependencies]
mwhash = "0.1.0"
```

---

## Examples

### One-shot hashing

Use this when all your data is already in memory.

```rust
use mwhash::mwhash;

let hash = mwhash(b"hello mwhash");
println!("Result: {:08x}", hash);

assert_ne!(hash, 0);
```

### Incremental hashing

Use this when data arrives in chunks — e.g. from a file or a network stream. No need to keep the whole buffer in memory.

```rust
use mwhash::{Hasher, mwhash};

let mut h = Hasher::new();
h.update(b"hello");
h.update(b" rust");

let hash = h.finish();
println!("Incremental hash: {:08x}", hash);

assert_eq!(hash, mwhash(b"hello rust"));
```

### Hashing with a numeric seed

The same input produces a different hash in a different "domain" — useful for salting data or separating hash spaces.

```rust
use mwhash::{mwhash, Hasher};

let mut h = Hasher::with_seed(0x12345678);
h.update(b"codeberg better than github");
let h1 = h.finish();

println!("Seeded hash: {:08x}", h1);
assert_ne!(h1, mwhash(b"codeberg better than github"));
```

### Hashing with a string seed

More convenient than picking magic numbers by hand — the string itself is hashed into a numeric seed.

```rust
use mwhash::{mwhash, Hasher};

let mut h = Hasher::with_string_seed("My-SID-is-definitely-unique");
h.update(b"100% essential data");
let hash = h.finish();

println!("String-seeded hash: {:08x}", hash);

assert_ne!(hash, 0);
assert_ne!(hash, mwhash(b"100% essential data"));
```

### One-shot hashing with a seed

The fastest way to get a seeded hash when you don't need to process data in chunks.

```rust
use mwhash::mwhash_seeded;

let data = b"test";
let seed = 0xDEAD_BEEF;

let hash = mwhash_seeded(data, seed);
println!("Hash: {:08x}", hash);

assert_ne!(hash, mwhash::mwhash(data));
```

### Concatenation without allocation

The order of updates doesn't matter — the result matches the hash of the combined data.

```rust
use mwhash::{mwhash, mwhash_concat};

let part1 = b"foo";
let part2 = b"bar";

let combined_hash = mwhash_concat(part1, part2);
let full_hash = mwhash(b"foobar");

println!("Hash: {:08x}", combined_hash);
assert_eq!(combined_hash, full_hash);
```

### Reusing the Hasher

`reset()` returns the hasher to its initial state without new allocations — useful when processing many independent messages.

```rust
use mwhash::{mwhash_seeded, Hasher};

let custom_seed = 0x12345678;
let mut h = Hasher::with_seed(custom_seed);

h.update(b"data1");
let hash1 = h.finish();
println!("Hash 1: {:08x}", hash1);

h.reset();
h.update(b"data2");
let hash2 = h.finish();
println!("Hash 2: {:08x}", hash2);

assert_eq!(hash2, mwhash_seeded(b"data2", custom_seed));
assert_ne!(hash1, hash2);
```

After `reset()`, a hasher with a custom seed behaves exactly as it did right after creation with that same seed:

```rust
use mwhash::Hasher;

let custom_seed = 0x12345678;
let mut h = Hasher::with_seed(custom_seed);

h.update(b"some data");
println!("Hash: {:08x}", h.finish());

h.reset();

assert_eq!(h.finish(), mwhash::mwhash_seeded(b"", custom_seed));

h.update(b"new data");
println!("New hash after reset: {:08x}", h.finish());
```

---

## no_std

The library is fully `no_std`, with no `alloc`. It runs on any target, including bare metal.

---

## License

Apache License 2.0. [View License](./LICENSE)
