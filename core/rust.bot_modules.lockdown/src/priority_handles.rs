/// Priority sets
///
/// PrioritySet can add elements with its priority with all priorities being saved.
/// When removing an element, the elements prior priority will be set.
use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub struct PrioritySet<T: std::hash::Hash + Clone + PartialEq + Eq> {
    elements: HashMap<T, VecDeque<usize>>,
}

/// Adds an element with a priority
///
/// If the element already exists, the priority will be added to the list of priorities.
/// If the element does not exist, a new list of priorities will be created.
impl<T: std::hash::Hash + Clone + PartialEq + Eq> PrioritySet<T> {
    pub fn add(&mut self, element: T, priority: usize) {
        if let Some(priorities) = self.elements.get_mut(&element) {
            priorities.push_back(priority);
        } else {
            let mut priorities = VecDeque::new();
            priorities.push_back(priority);
            self.elements.insert(element, priorities);
        }
    }

    /// Removes an element with a priority with priorities removed from back to front
    pub fn remove(&mut self, element: T, priority: usize) {
        if let Some(priorities) = self.elements.get_mut(&element) {
            let mut removed = false;
            let mut new_priorities = VecDeque::new();

            while let Some(p) = priorities.pop_back() {
                if p == priority && !removed {
                    // We've removed an element, so we need to skip this priority
                    removed = true;
                } else {
                    // Reinstate the priority in new_priorities
                    new_priorities.push_back(p);
                }
            }

            // If the element has no more priorities, remove it
            // Otherwise, update the element with the new priorities
            if new_priorities.is_empty() {
                self.elements.remove(&element);
            } else {
                self.elements.insert(element, new_priorities);
            }
        }
    }

    /// Finds the highest priority of an element
    /// Returns None if the element does not exist
    pub fn highest_priority(&self, element: &T) -> Option<usize> {
        if let Some(priorities) = self.elements.get(element) {
            priorities.iter().rev().max().cloned()
        } else {
            None
        }
    }

    /// Finds the lowest priority of an element
    /// Returns None if the element does not exist
    pub fn lowest_priority(&self, element: &T) -> Option<usize> {
        if let Some(priorities) = self.elements.get(element) {
            priorities.iter().min().cloned()
        } else {
            None
        }
    }

    /// Checks if an element exists
    /// Returns true if the element exists
    pub fn exists(&self, element: &T) -> bool {
        self.elements.contains_key(element)
    }

    /// Iterates over the elements and their highest priorities
    pub fn iter(&self) -> PrioritySetIter<T> {
        PrioritySetIter {
            iter: self.elements.iter(),
        }
    }
}

/// Iterates over the elements and their highest priorities
pub struct PrioritySetIter<'a, T: std::hash::Hash + Clone + PartialEq + Eq> {
    iter: std::collections::hash_map::Iter<'a, T, VecDeque<usize>>,
}

impl<'a, T: std::hash::Hash + Clone + PartialEq + Eq> Iterator for PrioritySetIter<'a, T> {
    type Item = (&'a T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(k, v)| (k, *v.iter().rev().max().unwrap()))
    }
}
