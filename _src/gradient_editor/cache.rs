/*
Adapted from: https://gitlab.com/polwel/egui-colorgradient/-/tree/master

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
*/

use std::any::Any;

use bevy_egui::egui::ahash::HashMap;
use bevy_egui::egui::util::cache::CacheTrait;

type Generation = u32;

pub struct FrameCacheDyn<Value, const TTL: Generation> {
    generation: Generation,
    cache: HashMap<u64, (Generation, Value)>,
}

impl<Value, const TTL: Generation> FrameCacheDyn<Value, TTL> {
    pub fn new() -> Self {
        Self {
            generation: 0,
            cache: Default::default(),
        }
    }

    /// Must be called once per frame to clear the cache.
    pub fn evict_cache(&mut self) {
        let current_generation = self.generation;
        self.cache.retain(|_key, (cached_generation, _cached_val)| {
            current_generation.wrapping_sub(*cached_generation) <= TTL // only keep those that were used recently
        });
        self.generation = self.generation.wrapping_add(1);
    }
}

impl<Value, const TTL: Generation> Default for FrameCacheDyn<Value, TTL> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Value, const TTL: Generation> FrameCacheDyn<Value, TTL> {
    /// Get from cache (if the same key was used last frame)
    /// or recompute and store in the cache.
    pub fn get_or_else_insert<Key, Computer>(&mut self, key: Key, computer: Computer) -> Value
    where
        Key: Copy + std::hash::Hash,
        Value: Clone,
        Computer: FnOnce() -> Value,
    {
        let hash = bevy_egui::egui::util::hash(key);

        match self.cache.entry(hash) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let cached = entry.into_mut();
                cached.0 = self.generation;
                cached.1.clone()
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = computer();
                entry.insert((self.generation, value.clone()));
                value
            }
        }
    }
}

impl<Value: 'static + Send + Sync, const TTL: Generation> CacheTrait for FrameCacheDyn<Value, TTL> {
    fn update(&mut self) {
        self.evict_cache()
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
