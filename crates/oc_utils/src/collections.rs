pub trait WithIds<I, T> {
    fn with_ids(&self) -> Vec<(I, T)>;
}

pub trait InvertedIndex<T> {
    fn get_r(&self, i: usize) -> Option<&T>;
}

impl<T> InvertedIndex<T> for Vec<T> {
    fn get_r(&self, i: usize) -> Option<&T> {
        let index = self.len() - 1 - i;
        self.get(index)
    }
}
