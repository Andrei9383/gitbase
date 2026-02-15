use crate::stores::Store;

pub struct DefaultStore {
    pub name: String,
}

impl Store for DefaultStore {
    fn insert(&self) {}
}
