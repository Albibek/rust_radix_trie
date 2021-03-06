use std::slice;
use std::iter::{Map, FilterMap, FromIterator};

use {Trie, TrieKey};

// MY EYES.
pub type Child<K, V> = Box<Trie<K, V>>;
pub type RawChildIter<'a, K, V> = slice::Iter<'a, Option<Child<K, V>>>;
pub type ChildMapFn<'a, K, V> = fn(&'a Option<Child<K, V>>) -> Option<&'a Child<K, V>>;
pub type ChildIter<'a, K, V> = FilterMap<RawChildIter<'a, K, V>, ChildMapFn<'a, K, V>>;

/// Iterator over the keys and values of a Trie.
pub struct Iter<'a, K: 'a, V: 'a> {
    root: &'a Trie<K, V>,
    root_visited: bool,
    stack: Vec<ChildIter<'a, K, V>>
}

impl<'a, K, V> Iter<'a, K, V> {
    pub fn new(root: &Trie<K, V>) -> Iter<K, V> {
        Iter {
            root: root,
            root_visited: false,
            stack: vec![]
        }
    }
}

/// Iterator over the keys of a Trie.
pub struct Keys<'a, K: 'a, V: 'a> {
    inner: Map<Iter<'a, K, V>, KeyMapFn<'a, K, V>>
}

type KeyMapFn<'a, K, V> = fn((&'a K, &'a V)) -> &'a K;

impl<'a, K, V> Keys<'a, K, V> {
    pub fn new(iter: Iter<'a, K, V>) -> Keys<'a, K, V> {
        fn first<'b, K, V>((k, _): (&'b K, &'b V)) -> &'b K { k }
        Keys { inner: iter.map(first) }
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<&'a K> {
        self.inner.next()
    }
}

/// Iterator over the values of a Trie.
pub struct Values<'a, K: 'a, V: 'a> {
    inner: Map<Iter<'a, K, V>, ValueMapFn<'a, K, V>>
}

type ValueMapFn<'a, K, V> = fn((&'a K, &'a V)) -> &'a V;

impl<'a, K, V> Values<'a, K, V> {
    pub fn new(iter: Iter<'a, K, V>) -> Values<'a, K, V> {
        fn second<'b, K, V>((_, v): (&'b K, &'b V)) -> &'b V { v }
        Values { inner: iter.map(second) }
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<&'a V> {
        self.inner.next()
    }
}

impl<K, V> Trie<K, V> {
    /// Helper function to get all the non-empty children of a node.
    pub fn child_iter<'a>(&'a self) -> ChildIter<'a, K, V> {
        fn id<'b, K, V>(x: &'b Option<Child<K, V>>) -> Option<&'b Child<K, V>> {
            x.as_ref()
        }

        self.children.iter().filter_map(id)
    }

    /// Get the key and value of a node as a pair.
    fn kv_as_pair<'a>(&'a self) -> Option<(&'a K, &'a V)> {
        self.key_value.as_ref().map(|kv| (&kv.key, &kv.value))
    }
}

enum IterAction<'a, K: 'a, V: 'a> {
    Push(&'a Trie<K, V>),
    Pop
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        use self::IterAction::*;

        // Visit each node as it is reached from its parent (with special root handling).
        if !self.root_visited {
            self.root_visited = true;
            self.stack.push(self.root.child_iter());
            if let Some(kv) = self.root.kv_as_pair() {
                return Some(kv);
            }
        }

        loop {
            let action = match self.stack.last_mut() {
                Some(stack_top) => {
                    match stack_top.next() {
                        Some(child) => Push(&child),
                        None => Pop
                    }
                }
                None => return None
            };

            match action {
                Push(trie) => {
                    self.stack.push(trie.child_iter());
                    if let Some(kv) = trie.kv_as_pair() {
                        return Some(kv);
                    }
                }
                Pop => { self.stack.pop(); }
            }
        }
    }
}

impl<K, V> FromIterator<(K, V)> for Trie<K, V> where K: TrieKey {
    fn from_iter<T>(iter: T) -> Trie<K, V> where T: IntoIterator<Item=(K, V)> {
        let mut trie = Trie::new();
        for (k, v) in iter {
            trie.insert(k, v);
        }
        trie
    }
}
