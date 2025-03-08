use std::collections::HashMap;
use handy::{Handle, HandleMap};
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

/// A trait for an object associated to a DBUS path.
pub trait DbusPath {
    fn path(&self) -> &OwnedObjectPath;
}

/// A store for objects associated to a DBUS path.
///
/// It supports fast lookups and removals by handle, and also by path.
#[derive(Debug)]
pub struct DbusStore<T> {
    /// DBUS path to handle, for fast lookup from a dbus path. Using `String` rather than
    /// `OwnedObjectPath` for convenience on lookup.
    path_to_handle: HashMap<String, Handle>,
    /// The actual storage for the items.
    map: HandleMap<T>,
}

impl<T: DbusPath> DbusStore<T> {
    pub fn insert(&mut self, el: T) -> Handle {
        let path = el.path().as_str().to_owned();
        let handle = self.map.insert(el);
        let old = self.path_to_handle.insert(path, handle);
        debug_assert!(old.is_none(), "Existing group with the same DBUS path?");
        handle
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        self.map.get(handle)
    }

    pub fn clear(&mut self) {
        self.path_to_handle.clear();
        self.map.clear();
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        self.map.get_mut(handle)
    }

    pub fn id_by_path(&self, path: &ObjectPath) -> Option<Handle> {
        self.path_to_handle.get(path.as_str()).cloned()
    }

    pub fn get_by_path(&self, path: &ObjectPath) -> Option<&T> {
        let element = self.get(self.id_by_path(path)?);
        debug_assert!(element.is_some(), "Found path, but not handle?");
        element
    }

    pub fn remove(&mut self, id: Handle) -> Option<T> {
        let el = self.map.remove(id)?;
        let handle = self.path_to_handle.remove(el.path().as_str());
        debug_assert_eq!(handle, Some(id), "Wrong path to handle association?");
        Some(el)
    }
}

impl<T> Default for DbusStore<T> {
    fn default() -> Self {
        Self {
            path_to_handle: Default::default(),
            map: Default::default(),
        }
    }
}
