//! Generational entity IDs and dense optional component storage.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityId {
    pub index: u32,
    pub generation: u32,
}

impl EntityId {
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: 0,
    };

    pub fn is_valid(self) -> bool {
        self.index != u32::MAX
    }
}

pub struct EntityAllocator {
    generations: Vec<u32>,
    free: Vec<u32>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn spawn(&mut self) -> EntityId {
        if let Some(index) = self.free.pop() {
            let generation = self.generations[index as usize];
            EntityId { index, generation }
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(0);
            EntityId {
                index,
                generation: 0,
            }
        }
    }

    pub fn despawn(&mut self, id: EntityId) -> bool {
        if !self.is_alive(id) {
            return false;
        }
        self.generations[id.index as usize] = self.generations[id.index as usize].wrapping_add(1);
        self.free.push(id.index);
        true
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        id.is_valid()
            && (id.index as usize) < self.generations.len()
            && self.generations[id.index as usize] == id.generation
    }

    pub fn capacity(&self) -> usize {
        self.generations.len()
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Dense slot storage keyed by entity index; invalid when generation mismatches.
pub struct DenseSlot<T> {
    slots: Vec<Option<(u32, T)>>,
}

impl<T> DenseSlot<T> {
    pub fn new() -> Self {
        Self { slots: Vec::new() }
    }

    pub fn insert(&mut self, id: EntityId, value: T) {
        let idx = id.index as usize;
        if idx >= self.slots.len() {
            self.slots.resize_with(idx + 1, || None);
        }
        self.slots[idx] = Some((id.generation, value));
    }

    pub fn remove(&mut self, id: EntityId) {
        if let Some(slot) = self.slots.get_mut(id.index as usize) {
            if let Some((gen, _)) = slot {
                if *gen == id.generation {
                    *slot = None;
                }
            }
        }
    }

    pub fn get(&self, id: EntityId) -> Option<&T> {
        self.slots
            .get(id.index as usize)
            .and_then(|s| s.as_ref())
            .and_then(|(gen, v)| (*gen == id.generation).then_some(v))
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        match slot {
            Some((gen, v)) if *gen == id.generation => Some(v),
            _ => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.slots.iter().enumerate().filter_map(|(i, s)| {
            s.as_ref().map(|(gen, v)| {
                (
                    EntityId {
                        index: i as u32,
                        generation: *gen,
                    },
                    v,
                )
            })
        })
    }
}

impl<T> Default for DenseSlot<T> {
    fn default() -> Self {
        Self::new()
    }
}
