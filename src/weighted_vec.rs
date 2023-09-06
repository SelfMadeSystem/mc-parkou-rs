use rand::prelude::*;

/// A weighted vec is a vec of elements with a weight associated with each element.
/// The weight is used to determine the probability of an element being selected.
#[derive(Debug, Clone)]
pub struct WeightedVec<T> {
    pub vec: Vec<(T, f32)>,
}

#[allow(dead_code)]
impl <T>WeightedVec<T> {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn push(&mut self, element: T, weight: f32) {
        self.vec.push((element, weight));
    }

    pub fn push_all(&mut self, elements: impl IntoIterator<Item = (T, f32)>) {
        self.vec.extend(elements);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.vec.get(index).map(|(element, _)| element)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vec.get_mut(index).map(|(element, _)| element)
    }

    pub fn get_weight(&self, index: usize) -> Option<f32> {
        self.vec.get(index).map(|(_, weight)| *weight)
    }

    pub fn remove_element(&mut self, element: &T) -> Option<T> where T: PartialEq {
        let index = self.vec.iter().position(|(e, _)| e == element)?;
        Some(self.vec.remove(index).0)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.vec.iter().map(|(element, _)| element)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.vec.iter_mut().map(|(element, _)| element)
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}

impl <T>WeightedVec<T> {
    pub fn get_random(&self) -> Option<&T> {
        let mut rng = rand::thread_rng();

        let total_weight = self.vec.iter().map(|(_, weight)| weight).sum();

        let mut random = rng.gen_range(0.0..total_weight);

        for (element, weight) in self.vec.iter() {
            random -= weight;
            if random <= 0.0 {
                return Some(element);
            }
        }

        None
    }
}

impl<T> std::ops::Index<usize> for WeightedVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index].0
    }
}

impl<T> std::ops::IndexMut<usize> for WeightedVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vec[index].0
    }
}

impl<T> std::iter::FromIterator<(T, f32)> for WeightedVec<T> {
    fn from_iter<I: IntoIterator<Item = (T, f32)>>(iter: I) -> Self {
        Self {
            vec: iter.into_iter().collect(),
        }
    }
}

impl<T> std::iter::Extend<(T, f32)> for WeightedVec<T> {
    fn extend<I: IntoIterator<Item = (T, f32)>>(&mut self, iter: I) {
        self.vec.extend(iter);
    }
}

/// Macro for creating a weighted vec.
///
/// # Example
/// ```
/// let weighted_vec = weighted_vec![
///    (1, 1.0),
///    (2, 2.0),
///    (3, 3.0),
/// ];
/// ```
#[macro_export]
macro_rules! weighted_vec {
    ($(($element:expr, $weight:expr)),* $(,)?) => {
        $crate::weighted_vec::WeightedVec::from_iter(vec![
            $(
                ($element, $weight),
            )*
        ])
    };
}