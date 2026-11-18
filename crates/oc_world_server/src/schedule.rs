use derive_more::Deref;

#[derive(Debug, Deref)]
pub struct Scheduling<T>(pub Vec<T>);

pub trait Schedule<C, T> {
    fn schedule(&self, ctx: C) -> Scheduling<T>;
}
