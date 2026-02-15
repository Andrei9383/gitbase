pub mod default_store;

pub trait Store {
    fn insert(&self);
}
