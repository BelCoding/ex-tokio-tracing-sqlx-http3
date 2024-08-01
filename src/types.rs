use std::net::SocketAddr;
extern crate derive_more;
use derive_more::{Display, From, FromStr};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

/// Email type comes in handy for the SQL query_as!. It should support String features.
#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, From, FromStr)]
pub struct Email {
    pub email: String,
}

impl Deref for Email {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.email
    }
}

impl DerefMut for Email {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.email
    }
}

/// Contact type comes in handy for the SQL query_as!.
#[derive(Debug)]
pub struct Contact {
    pub id: i32,
    pub email: Email,
    pub number: String,
}

/// EmailList with a HashSet efficient search and insert, to check the esistence of an email.
#[derive(Debug, From)]
pub struct EmailList {
    emails: HashSet<Email>,
}

impl Deref for EmailList {
    type Target = HashSet<Email>;

    fn deref(&self) -> &Self::Target {
        &self.emails
    }
}

impl DerefMut for EmailList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.emails
    }
}

impl std::iter::FromIterator<Email> for EmailList {
    fn from_iter<I: IntoIterator<Item = Email>>(iter: I) -> Self {
        let mut emails = HashSet::new();
        for email in iter {
            emails.insert(email);
        }
        EmailList { emails }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub email: Email,
    pub number: String,
}

/// Menu type to handle the operations via channels.
pub enum Menu {
    Add(Entry, SocketAddr),
    Get(Email, SocketAddr),
    // Remove(String), TODO!
    // List, Or GetList?
}

pub trait BufferOperations {
    fn from_n_bytes(bytes: [u8; 1024], len: usize) -> String;
    fn into_bytes(value: Self) -> Vec<u8>;
}

impl BufferOperations for String {
    fn from_n_bytes(bytes: [u8; 1024], len: usize) -> String {
        String::from_utf8(bytes[..len].to_vec()).expect("Invalid UTF-8")
    }

    fn into_bytes(value: Self) -> Vec<u8> {
        value.as_bytes().to_vec()
    }
}
