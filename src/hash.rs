#![allow(clippy::module_name_repetitions)]

use core::hash::BuildHasherDefault;

use rustc_hash::FxHasher;

pub type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;
pub type HashSet<T> = hashbrown::HashSet<T, BuildHasherDefault<FxHasher>>;

pub mod hash_map {
    use hashbrown::hash_map;

    pub type Iter<'a, K, V> = hash_map::Iter<'a, K, V>;
    pub type ExtractIf<'a, K, V, F> = hash_map::ExtractIf<'a, K, V, F>;
}
