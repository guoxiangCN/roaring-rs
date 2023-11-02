mod arbitrary;
mod fmt;
mod multiops;
mod proptests;

pub mod container;
pub mod store;
pub mod util;

// Order of these modules matters as it determines the `impl` blocks order in
// the docs
mod cmp;
mod inherent;
mod iter;
mod ops;
#[cfg(feature = "serde")]
mod serde;
mod serialization;

use std::ops::RangeBounds;

use self::cmp::Pairs;
use self::container::Container;
pub use self::iter::IntoIter;
pub use self::iter::Iter;
use self::util::join;

/// A compressed bitmap using the [Roaring bitmap compression scheme](https://roaringbitmap.org/).
///
/// # Examples
///
/// ```rust
/// use roaring::RoaringBitmap;
///
/// let mut rb = RoaringBitmap::new();
///
/// // insert all primes less than 10
/// rb.insert(2);
/// rb.insert(3);
/// rb.insert(5);
/// rb.insert(7);
/// println!("total bits set to true: {}", rb.len());
/// ```
#[derive(PartialEq)]
pub struct RoaringBitmap {
    containers: Vec<container::Container>,
}

impl RoaringBitmap {
    /// Allow user to access the roaring bitmap in container.
    pub fn foreach_container<F>(&self, f: F)
    where
        F: FnMut(&container::Container),
    {
        self.containers.iter().for_each(f)
    }

    /// Make the roaring bitmap from provided containers.
    pub fn from_containers(mut containers: Vec<container::Container>) -> Self {
        containers.sort_by_key(|c| c.key);
        Self { containers }
    }

    /// Find the first 1 within the given range.
    pub fn range_first1<R>(&self, range: R) -> Option<u32>
    where
        R: RangeBounds<u32>,
    {
        let (start, end) = match util::convert_range_to_inclusive(range) {
            Some(range) => (*range.start(), *range.end()),
            None => return None,
        };

        let (start_container_key, start_index) = util::split(start);
        let (end_container_key, end_index) = util::split(end);

        let mut index = 0;
        while index < self.containers.len() {
            let key = self.containers[index].key;
            if key >= start_container_key && key <= end_container_key {
                let begin = if key == start_container_key { start_index } else { 0_u16 };
                let end = if key == end_container_key { end_index } else { u16::MAX };
                if let Some(firstx) = self.containers[index].range_first1(begin..=end) {
                    return Some(join(key, firstx));
                }
            }
            index += 1;
        }

        None
    }

    /// Find the first 0 within the given range.
    pub fn range_first0<R>(&self, range: R) -> Option<u32>
    where
        R: RangeBounds<u32>,
    {
        let (start, end) = match util::convert_range_to_inclusive(range) {
            Some(range) => (*range.start(), *range.end()),
            None => return None,
        };

        let (start_container_key, start_index) = util::split(start);
        let (end_container_key, end_index) = util::split(end);

        let mut empty_container = Container::new(0);
        let mut container_key = start_container_key;
        while container_key <= end_container_key {
            let container_ref = match self.containers.binary_search_by_key(&container_key, |c|c.key) {
                Ok(idx) => &self.containers[idx],
                Err(_) => {
                    empty_container.key = container_key;
                    &empty_container
                },
            };

            if let Some(idx) = container_ref.range_first0(start_index..=end_index) {
                return Some(join(container_key, idx));
            }
            
            container_key += 1;
        }

        None
    }
}

mod tests {
    

    #[test]
    fn test_vector_binary_search() {
        let offsets = vec![100,200,300,400,500,600];
        match offsets.binary_search(&800) {
            Ok(idx) => println!("ok, idx={}", idx),
            Err(idx) => println!("err, idx={}", idx),
        }
    }
}
