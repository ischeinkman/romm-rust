use std::hash::Hash;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum FlattenedList<T> {
    #[default]
    None,
    Single(T),
    Multiple(Vec<T>),
}

impl<T> FlattenedList<T> {
    pub fn as_slice(&self) -> &[T] {
        match self {
            FlattenedList::None => &[],
            FlattenedList::Multiple(v) => v.as_slice(),
            FlattenedList::Single(itm) => std::slice::from_ref(itm),
        }
    }
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (FlattenedList::None, ret) | (ret, FlattenedList::None) => ret,
            (FlattenedList::Single(a), FlattenedList::Single(b)) => {
                FlattenedList::Multiple(vec![a, b])
            }
            (FlattenedList::Multiple(mut a), FlattenedList::Multiple(b)) => {
                a.extend(b);
                FlattenedList::Multiple(a)
            }
            (FlattenedList::Multiple(mut v), FlattenedList::Single(end)) => {
                v.push(end);
                FlattenedList::Multiple(v)
            }
            (FlattenedList::Single(start), FlattenedList::Multiple(mut v)) => {
                v.insert(0, start);
                FlattenedList::Multiple(v)
            }
        }
    }
}

impl<T> From<FlattenedList<T>> for Vec<T> {
    fn from(value: FlattenedList<T>) -> Self {
        match value {
            FlattenedList::None => Vec::new(),
            FlattenedList::Multiple(v) => v,
            FlattenedList::Single(itm) => vec![itm],
        }
    }
}

impl<T> Hash for FlattenedList<T>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        T::hash_slice(self.as_slice(), state);
    }
}

impl<T> PartialEq for FlattenedList<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T> Eq for FlattenedList<T> where T: Eq {}