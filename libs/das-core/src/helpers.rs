use core::ops::Deref;

use molecule::prelude::Entity;

#[derive(Debug)]
pub struct Comparable<T>(pub T);

impl<T> Deref for Comparable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> PartialOrd for Comparable<T>
where
    T: Entity,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T> Ord for Comparable<T>
where
    T: Entity,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T> PartialEq for Comparable<T>
where
    T: Entity,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T> Eq for Comparable<T> where T: Entity {}

impl<T> Clone for Comparable<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
