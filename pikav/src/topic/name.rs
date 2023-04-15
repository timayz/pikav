//! Topic name

use std::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

const TOPIC_NAME_VALIDATE_REGEX: &str = r"^[^#+]+$";

lazy_static! {
    static ref TOPIC_NAME_VALIDATOR: Regex = Regex::new(TOPIC_NAME_VALIDATE_REGEX).unwrap();
}

#[inline]
fn is_invalid_topic_name(topic_name: &str) -> bool {
    topic_name.is_empty()
        || topic_name.as_bytes().len() > 65535
        || !TOPIC_NAME_VALIDATOR.is_match(topic_name)
}

/// Topic name
///
/// http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106
#[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
pub struct TopicName(String);

impl TopicName {
    /// Creates a new topic name from string
    /// Return error if the string is not a valid topic name
    pub fn new<S: Into<String>>(topic_name: S) -> Result<TopicName, TopicNameError> {
        let topic_name = topic_name.into();
        if is_invalid_topic_name(&topic_name) {
            Err(TopicNameError(topic_name))
        } else {
            Ok(TopicName(topic_name))
        }
    }

    /// Creates a new topic name from string without validation
    ///
    /// # Safety
    ///
    /// Topic names' syntax is defined in [MQTT specification](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106).
    /// Creating a name from raw string may cause errors
    pub unsafe fn new_unchecked(topic_name: String) -> TopicName {
        TopicName(topic_name)
    }
}

impl From<TopicName> for String {
    fn from(topic_name: TopicName) -> String {
        topic_name.0
    }
}

impl Deref for TopicName {
    type Target = TopicNameRef;

    fn deref(&self) -> &TopicNameRef {
        unsafe { TopicNameRef::new_unchecked(&self.0) }
    }
}

impl DerefMut for TopicName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { TopicNameRef::new_mut_unchecked(&mut self.0) }
    }
}

impl Borrow<TopicNameRef> for TopicName {
    fn borrow(&self) -> &TopicNameRef {
        Deref::deref(self)
    }
}

impl BorrowMut<TopicNameRef> for TopicName {
    fn borrow_mut(&mut self) -> &mut TopicNameRef {
        DerefMut::deref_mut(self)
    }
}

impl Serialize for TopicName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for TopicName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(Deserialize::deserialize(deserializer)?))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid topic name ({0})")]
pub struct TopicNameError(pub String);

/// Reference to a topic name
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct TopicNameRef(str);

impl TopicNameRef {
    /// Creates a new topic name from string
    /// Return error if the string is not a valid topic name
    pub fn new<S: AsRef<str> + ?Sized>(topic_name: &S) -> Result<&TopicNameRef, TopicNameError> {
        let topic_name = topic_name.as_ref();
        if is_invalid_topic_name(topic_name) {
            Err(TopicNameError(topic_name.to_owned()))
        } else {
            Ok(unsafe { &*(topic_name as *const str as *const TopicNameRef) })
        }
    }

    /// Creates a new topic name from string
    /// Return error if the string is not a valid topic name
    pub fn new_mut<S: AsMut<str> + ?Sized>(
        topic_name: &mut S,
    ) -> Result<&mut TopicNameRef, TopicNameError> {
        let topic_name = topic_name.as_mut();
        if is_invalid_topic_name(topic_name) {
            Err(TopicNameError(topic_name.to_owned()))
        } else {
            Ok(unsafe { &mut *(topic_name as *mut str as *mut TopicNameRef) })
        }
    }

    /// Creates a new topic name from string without validation
    ///
    /// # Safety
    ///
    /// Topic names' syntax is defined in [MQTT specification](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106).
    /// Creating a name from raw string may cause errors
    pub unsafe fn new_unchecked<S: AsRef<str> + ?Sized>(topic_name: &S) -> &TopicNameRef {
        let topic_name = topic_name.as_ref();
        &*(topic_name as *const str as *const TopicNameRef)
    }

    /// Creates a new topic name from string without validation
    ///
    /// # Safety
    ///
    /// Topic names' syntax is defined in [MQTT specification](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106).
    /// Creating a name from raw string may cause errors
    pub unsafe fn new_mut_unchecked<S: AsMut<str> + ?Sized>(
        topic_name: &mut S,
    ) -> &mut TopicNameRef {
        let topic_name = topic_name.as_mut();
        &mut *(topic_name as *mut str as *mut TopicNameRef)
    }

    /// Check if this topic name is only for server.
    ///
    /// Topic names that beginning with a '$' character are reserved for servers
    pub fn is_server_specific(&self) -> bool {
        self.0.starts_with('$')
    }
}

impl Deref for TopicNameRef {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl ToOwned for TopicNameRef {
    type Owned = TopicName;

    fn to_owned(&self) -> Self::Owned {
        TopicName(self.0.to_owned())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn topic_name_sys() {
        let topic_name = "$SYS".to_owned();
        TopicName::new(topic_name).unwrap();

        let topic_name = "$SYS/broker/connection/test.cosm-energy/state".to_owned();
        TopicName::new(topic_name).unwrap();
    }

    #[test]
    fn topic_name_slash() {
        TopicName::new("/").unwrap();
    }

    #[test]
    fn topic_name_basic() {
        TopicName::new("/finance").unwrap();
        TopicName::new("/finance//def").unwrap();
    }
}
