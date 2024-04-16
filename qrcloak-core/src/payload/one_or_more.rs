use core::slice;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Inner<'a, T> {
    Borrowed(&'a [T]),
    Owned(Vec<T>),
}

#[cfg(feature = "serde")]
impl<'a, T: serde::Serialize> serde::Serialize for Inner<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            Inner::Borrowed(value) => value.serialize(serializer),
            Inner::Owned(value) => value.serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
impl<'a, 'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Inner<'a, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Vec::<T>::deserialize(deserializer)?;

        if value.is_empty() {
            return Err(serde::de::Error::custom("expected at least one element"));
        }

        Ok(Inner::Owned(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct OneOrMore<'a, T>(Inner<'a, T>);

impl<'a, T> OneOrMore<'a, T> {
    pub fn is_one(&self) -> bool {
        match &self.0 {
            Inner::Borrowed(value) => value.len() == 1,
            Inner::Owned(value) => value.len() == 1,
        }
    }

    pub fn first(&self) -> &T {
        match &self.0 {
            Inner::Borrowed(value) => value.first().unwrap(),
            Inner::Owned(value) => value.first().unwrap(),
        }
    }

    pub fn is_many(&self) -> bool {
        !self.is_one()
    }

    pub fn as_slice(&self) -> &[T] {
        match &self.0 {
            Inner::Borrowed(value) => value,
            Inner::Owned(value) => value.as_slice(),
        }
    }
}

impl<'a, T> Into<Inner<'a, T>> for OneOrMore<'a, T> {
    fn into(self) -> Inner<'a, T> {
        self.0
    }
}

impl<'a, T> From<Inner<'a, T>> for OneOrMore<'a, T> {
    fn from(value: Inner<'a, T>) -> Self {
        Self(value)
    }
}

impl<'a, T> TryFrom<Vec<T>> for OneOrMore<'a, T> {
    type Error = Vec<T>;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(value);
        }

        Ok(Self(Inner::Owned(value)))
    }
}

impl<'a, T> TryFrom<&'a [T]> for OneOrMore<'a, T> {
    type Error = &'a [T];

    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(value);
        }

        Ok(Self(Inner::Borrowed(value)))
    }
}

impl<'a, T> From<&'a T> for OneOrMore<'a, T> {
    fn from(value: &'a T) -> Self {
        Self(Inner::Borrowed(slice::from_ref(value)))
    }
}

impl<'a, T> From<T> for OneOrMore<'a, T> {
    fn from(value: T) -> Self {
        Self(Inner::Owned(vec![value]))
    }
}

impl<'a, T> AsRef<[T]> for OneOrMore<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T> AsRef<OneOrMore<'a, T>> for OneOrMore<'a, T> {
    fn as_ref(&self) -> &OneOrMore<'a, T> {
        self
    }
}
