# mwhash

Быстрый некриптографический хеш `u32`. `no_std`, без зависимостей, без аллокаций.

Подходит для: проверки целостности пакетов, контрольных сумм, быстрого сравнения буферов, хеш-таблиц, embedded-проектов.
Не подходит для криптографии.

---

## Производительность

Ниже приведены результаты бенчмарков (seed = `0x12345678`):

```text
----------------------------------------------------------------------------------------------------
64 KB      | size:     64.00 KB  | iters:  20000 | total: 339.070ms | per call:   16.953µs | throughput:   3.87 GB/s
1 MB       | size:   1024.00 KB  | iters:   2000 | total: 549.020ms | per call:  274.509µs | throughput:   3.82 GB/s
1 GB       | size: 1048576.00 KB | iters:      3 | total: 979.626ms | per call:  326.542ms | throughput:   3.29 GB/s
----------------------------------------------------------------------------------------------------
```

* **64 КБ:** кэш-память L1/L2.
* **1 МБ:**  кэш-память L3.
* **1 ГБ:**  основная память.

> **Окружающая среда:*** Тест проводился на архитектуре АМД64 (Процессор: [Intel core ultra 5 125h], ОЗУ: [16ГБ lpddr5x 6000MT/s]).
> **Исходный код:** [examples/benchmark.rs](./examples/benchmark.rs)

---

## Установка

```toml
[dependencies]
mwhash = "0.1.0"
```

---

## Примеры

### Одноразовый хеш

Когда все данные уже доступны целиком.

```rust
use mwhash::mwhash;

let hash = mwhash(b"hello mwhash");
println!("Result: {:08x}", hash);

assert_ne!(hash, 0);
```

### Инкрементальный хеш

Когда данные приходят по частям — например, из файла или сети. Не требует хранить весь буфер в памяти.

```rust
use mwhash::{Hasher, mwhash};

let mut h = Hasher::new();
h.update(b"hello");
h.update(b" rust");

let hash = h.finish();
println!("Incremental hash: {:08x}", hash);

assert_eq!(hash, mwhash(b"hello rust"));
```

### Хеш с числовым seed

Один и тот же вход даёт разные хеши в разных "доменах" — удобно для солинга данных или разделения хеш-пространств.

```rust
use mwhash::{mwhash, Hasher};

let mut h = Hasher::with_seed(0x12345678);
h.update(b"codeberg better than github");
let h1 = h.finish();

println!("Seeded hash: {:08x}", h1);
assert_ne!(h1, mwhash(b"codeberg better than github"));
```

### Хеш со строковым seed

Удобнее, чем подбирать magic-числа вручную — строка сама хешируется в числовой seed.

```rust
use mwhash::{mwhash, Hasher};

let mut h = Hasher::with_string_seed("My-SID-is-definitely-unique");
h.update(b"100% essential data");
let hash = h.finish();

println!("String-seeded hash: {:08x}", hash);

assert_ne!(hash, 0);
assert_ne!(hash, mwhash(b"100% essential data"));
```

### Одноразовый хеш с seed

Самый быстрый способ получить хеш с seed, когда данные не нужно обрабатывать по частям.

```rust
use mwhash::mwhash_seeded;

let data = b"test";
let seed = 0xDEAD_BEEF;

let hash = mwhash_seeded(data, seed);
println!("Hash: {:08x}", hash);

assert_ne!(hash, mwhash::mwhash(data));
```

### Конкатенация без аллокации

Порядок добавления частей не влияет на результат — итог совпадает с хешем объединённых данных.

```rust
use mwhash::{mwhash, mwhash_concat};

let part1 = b"foo";
let part2 = b"bar";

let combined_hash = mwhash_concat(part1, part2);
let full_hash = mwhash(b"foobar");

println!("Hash: {:08x}", combined_hash);
assert_eq!(combined_hash, full_hash);
```

### Повторное использование Hasher

`reset()` возвращает хешер в исходное состояние без новых выделений памяти — удобно при обработке множества независимых сообщений.

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

После `reset()` хешер с custom seed ведёт себя так же, как сразу после создания с этим же seed:

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

Библиотека полностью `no_std`, без `alloc`. Работает на любом таргете, включая bare-metal.

---

## Лицензия

Apache License 2.0. [Посмотреть](./LICENSE)
