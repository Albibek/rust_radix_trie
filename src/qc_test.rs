//! Proper testing, with QuickCheck.

use {Trie, TrieKey};
use std::iter::FromIterator;
use std::collections::{HashSet, HashMap};
use quickcheck::{quickcheck, Gen, Arbitrary};
use rand::Rng;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key(Vec<u8>);

#[derive(Clone, Debug)]
struct RandomKeys(HashSet<Key>);

const MAX_KEYS: usize = 512;
const KEY_RUN_LEN: usize = 8;
const KEY_MAX_VAL: u8 = 4;

impl Arbitrary for Key {
    fn arbitrary<G: Gen>(g: &mut G) -> Key {
        let len = g.gen::<usize>() % KEY_RUN_LEN;
        let mut key = Vec::with_capacity(len);
        for _ in 0 .. len {
            key.push(g.gen::<u8>() % KEY_MAX_VAL);
        }
        Key(key)
    }
}

impl Key {
    fn extend_random<G: Gen>(&self, g: &mut G) -> Key {
        let mut key = self.clone();
        key.0.extend(Key::arbitrary(g).0);
        key
    }

    fn len(&self) -> usize { self.0.len() }
}

impl TrieKey for Key {
    fn encode(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl Arbitrary for RandomKeys {
    fn arbitrary<G: Gen>(g: &mut G) -> RandomKeys {
        let num_keys = g.gen::<usize>() % MAX_KEYS;
        let mut keys = Vec::with_capacity(num_keys);
        keys.push(Key::arbitrary(g));

        for _ in 0 .. num_keys {
            match g.gen::<u8>() % 10 {
                // Generate a new random key.
                1 => keys.push(Key::arbitrary(g)),
                // Extend an existing key.
                _ => {
                    let i = g.gen::<usize>() % keys.len();
                    let key = keys[i].extend_random(g);
                    keys.push(key);
                }
            }
        }

        RandomKeys(HashSet::from_iter(keys))
    }
}

#[test]
fn insert_all_remove_all() {
    fn prop(RandomKeys(keys): RandomKeys) -> bool {
        let mut trie = Trie::new();
        let mut length = 0;

        for k in &keys {
            if trie.insert(k.clone(), k.len()).is_some() {
                return false;
            }
            length += 1;
            if trie.len() != length { return false }
        }

        if !trie.check_integrity() { return false }

        for k in &keys {
            if trie.get(&k) != Some(&k.len()) {
                return false;
            }
            if trie.remove(&k) != Some(k.len()) {
                return false;
            }
            length -= 1;
            if trie.len() != length { return false }
            if trie.get(&k).is_some() { return false }
        }
        if !trie.check_integrity() { return false }
        true
    }

    quickcheck(prop as fn(RandomKeys) -> bool);
}

#[test]
fn get_node() {
    fn prop(RandomKeys(keys): RandomKeys) -> bool {
        let half = keys.len() / 2;
        let first_half = keys.iter().take(half).map(|k| (k.clone(), k.len()));
        let trie = Trie::from_iter(first_half);

        // Check node existence for inserted keys.
        for k in keys.iter().take(half) {
            match trie.get_node(&k) {
                Some(node) => if node.value() != Some(&k.len()) { return false },
                None => return false
            }
        }

        // Check that nodes for non-inserted keys don't have values.
        for k in keys.iter().skip(half) {
            if let Some(node) = trie.get_node(&k) {
                if node.value().is_some() { return false }
            }
        }

        true
    }

    quickcheck(prop as fn(RandomKeys) -> bool);
}

// Construct a trie from a set of keys, with each key mapped to its length.
fn length_trie(keys: HashSet<Key>) -> Trie<Key, usize> {
    let mut t = Trie::new();
    for k in keys {
        let len = k.len();
        t.insert(k, len);
    }
    t
}

#[test]
fn keys_iter() {
    fn prop(RandomKeys(keys): RandomKeys) -> bool {
        let trie = length_trie(keys.clone());
        let trie_keys: HashSet<Key> = trie.keys().cloned().collect();
        trie_keys == keys
    }
    quickcheck(prop as fn(RandomKeys) -> bool);
}

#[test]
fn values_iter() {
    // Create a map of values to frequencies.
    fn frequency_map<I: Iterator<Item=usize>>(values: I) -> HashMap<usize, u64> {
        let mut map = HashMap::new();
        for v in values {
            let current_val = map.entry(v).or_insert(0);
            *current_val += 1;
        }
        map
    }

    fn prop(RandomKeys(keys): RandomKeys) -> bool {
        let trie = length_trie(keys.clone());
        let trie_values: HashMap<usize, u64> = frequency_map(trie.values().cloned());
        let key_values = frequency_map(keys.into_iter().map(|k| k.len()));
        trie_values == key_values
    }
    quickcheck(prop as fn(RandomKeys) -> bool);
}
