pub trait Monoid {
    fn m_empty() -> Self;
    fn m_append(&self, other: &Self) -> Self;
}
