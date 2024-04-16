use std::{borrow::Cow, ops::Deref, slice};

use crate::format::{CompletePayload, PartialPayload, Payload};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OneOrMore<'a, T: Clone>(Cow<'a, [T]>);

#[cfg(feature = "serde")]
impl<'a, T: serde::Serialize + Clone> serde::Serialize for OneOrMore<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.0.len() == 1 {
            self.0[0].serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

#[cfg(feature = "serde")]
impl<'a, 'de, T: serde::Deserialize<'de> + Clone> serde::Deserialize<'de> for OneOrMore<'a, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum OneOrMany<T> {
            One(T),
            Many(Vec<T>),
        }

        match OneOrMany::<T>::deserialize(deserializer)? {
            OneOrMany::One(value) => Ok(OneOrMore(Cow::Owned(vec![value]))),
            OneOrMany::Many(value) => Ok(OneOrMore(Cow::Owned(value))),
        }
    }
}

impl<'a, T: Clone> OneOrMore<'a, T> {
    pub fn is_one(&self) -> bool {
        self.0.len() == 1
    }

    pub fn first(&self) -> &T {
        &self.0[0]
    }

    pub fn is_many(&self) -> bool {
        !self.is_one()
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.0.iter()
    }
}

impl<'a, T: Clone> Deref for OneOrMore<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Clone> IntoIterator for OneOrMore<'a, T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_owned().into_iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a OneOrMore<'a, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T: Clone> From<T> for OneOrMore<'a, T> {
    fn from(value: T) -> Self {
        Self(Cow::Owned(vec![value]))
    }
}

impl<'a, T: Clone> TryFrom<Vec<T>> for OneOrMore<'a, T> {
    type Error = Vec<T>;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(value);
        }

        Ok(Self(Cow::Owned(value)))
    }
}

impl<'a, T: Clone> TryFrom<&'a [T]> for OneOrMore<'a, T> {
    type Error = &'a [T];

    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(value);
        }

        Ok(Self(Cow::Borrowed(value)))
    }
}

impl<'a, T: Clone> From<&'a T> for OneOrMore<'a, T> {
    fn from(value: &'a T) -> Self {
        Self(Cow::Borrowed(slice::from_ref(value)))
    }
}

impl<'a, T: Clone> Into<Vec<T>> for OneOrMore<'a, T> {
    fn into(self) -> Vec<T> {
        self.0.into_owned()
    }
}

impl<'a> OneOrMore<'a, PartialPayload> {
    pub fn into_payloads(self) -> OneOrMore<'a, Payload> {
        self.0
            .into_owned()
            .into_iter()
            .map(|p| p.into())
            .collect::<Vec<_>>()
            .try_into()
            .expect("at least one element")
    }
}

impl<'a> OneOrMore<'a, CompletePayload> {
    pub fn into_payloads(self) -> OneOrMore<'a, Payload> {
        self.0
            .into_owned()
            .into_iter()
            .map(|p| p.into())
            .collect::<Vec<_>>()
            .try_into()
            .expect("at least one element")
    }
}

impl<'a> OneOrMore<'a, Payload> {
    pub fn split(self) -> (Vec<CompletePayload>, Vec<PartialPayload>) {
        let mut partials = Vec::new();
        let mut completes = Vec::new();

        for payload in self.into_iter() {
            match payload {
                Payload::Complete(c) => completes.push(c),
                Payload::Partial(p) => partials.push(p),
            }
        }

        (completes, partials)
    }
}
