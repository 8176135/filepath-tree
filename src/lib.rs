mod errors;

use errors::StorageError;

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::sync::{RwLock};
use std::hash::Hash;

type PathNodeRef<T> = Rc<RwLock<PathNode<T>>>;
type PathNodeRefWeak<T> = Weak<RwLock<PathNode<T>>>;

struct PathNode<T> {
	name: OsString,
	data: Option<T>,
	items: HashMap<OsString, PathNodeRef<T>>,
	parent: Option<PathNodeRefWeak<T>>,
}

impl<T> PathNode<T> {
	/// Creates the root path node
	pub fn root(data: Option<T>) -> Self {
		Self {
			name: OsString::from("/"),
			items: HashMap::new(),
			data,
			parent: None,
		}
	}

	pub fn new(name: OsString, data: Option<T>, parent: PathNodeRefWeak<T>) -> Self {
		Self {
			name,
			items: HashMap::new(),
			data,
			parent: Some(parent),
		}
	}

	pub fn set_data(&mut self, data: Option<T>) {
		self.data = data;
	}
}

pub struct PathStore<T> {
	root: PathNodeRef<T>,
	size: usize,
}

impl<T> PathStore<T> {
	pub fn new(data: Option<T>) -> Self {
		Self {
			root: Rc::new(RwLock::new(PathNode::root(data))),
			size: 0,
		}
	}

	/// Add path, returns true if it was not already in the store
	///
	/// The added path must be absolute
	pub fn add_path<P: AsRef<Path>>(&mut self, path: P, data: Option<T>) -> Result<bool, StorageError> {
		if !path.as_ref().is_absolute() {
			return Err(StorageError::PathNotRelative);
		}

		let mut comp = path.as_ref().components().skip(1); // Skip the root path
		let mut current_in_tree = self.root.clone();

		let mut changed = false;

		while let Some(item) = comp.next() {
			let mut current_tree_lock = current_in_tree
				.read()
				.expect("Failed to lock tree node when adding path");
			if let Some(c) = current_tree_lock.items.get(item.as_os_str()) {
				let c = c.clone();
				drop(current_tree_lock);
				current_in_tree = c.clone();
			} else {
				self.size += 1;
				changed = true;
				let to_add = Rc::new(RwLock::new(PathNode::new(
					item.as_os_str().to_os_string(),
					None,
					Rc::downgrade(&current_in_tree),
				)));

				drop(current_tree_lock);
				{
					let mut current_write_lock = current_in_tree.write().unwrap();
					current_write_lock
						.items
						.insert(item.as_os_str().to_os_string(), to_add.clone());
				}
				current_in_tree = to_add;
			}
		}
		current_in_tree.write().unwrap().set_data(data);
		Ok(changed)
	}

	pub fn walk(&self) -> Vec<OsString> {
		let mut out = Vec::new();
		Self::walk_inner(&self.root, &mut PathBuf::new(), &mut out);
		out
	}

	fn walk_inner(current_node: &PathNodeRef<T>, current_dir: &mut PathBuf, out: &mut Vec<OsString>) {
		let mut current_node = &current_node
			.read()
			.expect("Failed to lock tree node when adding path");

		current_dir.push(&current_node.name);

		if current_node.items.is_empty() {
			out.push(current_dir.as_os_str().to_owned());
//			println!("{}", current_dir.display())
		} else {
			for item in current_node.items.values() {
				Self::walk_inner(item, current_dir, out);
			}
		}

		current_dir.pop();
	}

	pub fn size(&self) -> usize {
		self.size
	}
}

#[cfg(test)]
mod tests {
	use super::PathStore;

	#[test]
	fn root_store_push() {
		let mut store = PathStore::new();
		assert_eq!(store.size, 0);

		assert_eq!(store.add_path("/f"), Ok(true));
		assert_eq!(store.add_path("/g"), Ok(true));
		assert_eq!(store.add_path("/f"), Ok(false));
		assert_eq!(store.add_path("h").is_err(), true);
		assert_eq!(store.size, 2);
	}

	#[test]
	fn root_store_push_double() {
		let mut store = PathStore::new();
		assert_eq!(store.size, 0);

		assert_eq!(store.add_path("/f"), Ok(true));
		assert_eq!(store.add_path("/g"), Ok(true));
		assert_eq!(store.add_path("/f/FDrive/files"), Ok(true));
		assert_eq!(store.add_path("/f/FDrive/hello"), Ok(true));
		assert_eq!(store.add_path("/f"), Ok(false));
		assert_eq!(store.add_path("h").is_err(), true);
		assert_eq!(store.size, 5);

		dbg!(store.walk());
		panic!()
	}
}
