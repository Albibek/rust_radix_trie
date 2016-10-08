use {Trie, TrieNode, TrieKey, SubTrie, SubTrieMut, NibbleVec};

impl<K, V> Trie<K, V> where K: TrieKey {
    /// Create an empty Trie.
    pub fn new() -> Trie<K, V> {
        Trie {
            length: 0,
            node: TrieNode::new(),
        }
    }

    /// Fetch a reference to the given key's corresponding value, if any.
    pub fn get(&self, key: &K) -> Option<&V> {
        let key_fragments = key.encode();
        self.node.get(&key_fragments).and_then(|t| t.value_checked(key))
    }

    /// Fetch a mutable reference to the given key's corresponding value, if any.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let key_fragments = key.encode();
        self.node.get_mut(&key_fragments).and_then(|t| t.value_checked_mut(key))
    }

    /// Insert the given key-value pair, returning any previous value associated with the key.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key_fragments = key.encode();
        let result = self.node.insert(key, value, key_fragments);
        if result.is_none() {
            self.length += 1;
        }
        result
    }

    /// Remove the value associated with the given key.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let removed = self.node.remove(key);
        if removed.is_some() {
            self.length -= 1;
        }
        removed
    }

    /// Get a mutable reference to the value stored at this node, if any.
    pub fn value_mut(&mut self) -> Option<&mut V> {
        self.node.value_mut()
    }

    /// Fetch a reference to the subtrie for a given key.
    pub fn subtrie<'a>(&'a self, key: &K) -> Option<SubTrie<'a, K, V>> {
        let key_fragments = key.encode();
        self.node.get(&key_fragments).map(|node| {
            new_subtrie(key_fragments, node)
        })
    }

    /// Fetch a mutable reference to the subtrie for a given key.
    pub fn subtrie_mut<'a>(&'a mut self, key: &K) -> Option<SubTrieMut<'a, K, V>> {
        let key_fragments = key.encode();
        let length_ref = &mut self.length;
        self.node.get_mut(&key_fragments).map(move |node| {
            new_subtrie_mut(key_fragments, length_ref, node)
        })
    }

    /// Fetch a reference to the closest ancestor node of the given key.
    ///
    /// If `key` is encoded as byte-vector `b`, return the node `n` in the tree
    /// such that `n`'s key's byte-vector is the longest possible prefix of `b`, and `n`
    /// has a value.
    ///
    /// Invariant: `result.is_some() => result.key_value.is_some()`.
    pub fn get_ancestor<'a>(&'a self, key: &K) -> Option<SubTrie<'a, K, V>> {
        let key_fragments = key.encode();
        self.node.get_ancestor(&key_fragments).map(|node| {
            new_subtrie(key_fragments, node)
        })
    }

    /// Fetch the closest ancestor *value* for a given key.
    ///
    /// See `get_ancestor` for precise semantics, this is just a shortcut.
    pub fn get_ancestor_value(&self, key: &K) -> Option<&V> {
        self.get_ancestor(key).and_then(|t| t.node.value())
    }

    // FIXME
    /*
    pub fn get_raw_ancestor(&self, key: &K) -> &TrieNode<K, V> {
        GetRawAncestor::run(self, (), key.encode()).unwrap()
    }
    */

    /*
    /// Fetch the closest descendant for a given key.
    ///
    /// If the key is in the trie, this is the same as `get_node`.
    pub fn get_descendant<'a>(&self, key: &K) -> Option<SubTrie<'a, K, V>> {
        // FIXME:
        // let key_fragments = key.encode();
        // GetDescendant::run(self, (), key_fragments)
        None
    }
    */

    /// Take a function `f` and apply it to the value stored at `key`.
    ///
    /// If no value is stored at `key`, store `default`.
    pub fn map_with_default<F>(&mut self, key : K, f : F, default: V) where F: Fn(&mut V) {
        {
            if let Some(v) = self.get_mut(&key) {
                f(v);
                return;
            }
        }
        self.insert(key, default);
    }

    /// Check that the Trie invariants are satisfied - you shouldn't ever have to call this!
    /// Quite slow!
    #[doc(hidden)]
    pub fn check_integrity(&self) -> bool {
        let (ok, length) = self.node.check_integrity_recursive(&NibbleVec::new());
        ok && length == self.length
    }
}

// TODO: may as well make these public methods.
fn new_subtrie<'a, K, V>(prefix: NibbleVec, node: &'a TrieNode<K, V>) -> SubTrie<'a, K, V>
    where K: TrieKey
{
    SubTrie {
        prefix: prefix,
        node: node,
    }
}

fn new_subtrie_mut<'a, K, V>(prefix: NibbleVec, length: &'a mut usize, node: &'a mut TrieNode<K, V>)
    -> SubTrieMut<'a, K, V> where K: TrieKey
{
    SubTrieMut {
        prefix: prefix,
        length: length,
        node: node,
    }
}