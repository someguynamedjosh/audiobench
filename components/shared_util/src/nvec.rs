#[derive(Clone, Debug, PartialEq)]
pub struct NVec<T> {
    dimensions: Vec<usize>,
    multipliers: Vec<usize>,
    data: Vec<T>,
}

impl<T: Clone> NVec<T> {
    fn make_multipliers(dimensions: &[usize]) -> Vec<usize> {
        let mut result = Vec::new();
        for index in 0..dimensions.len() {
            let mut multiplier = 1;
            for dim in dimensions[index + 1..].iter() {
                multiplier *= *dim;
            }
            result.push(multiplier);
        }
        result
    }

    pub fn build_ok<E>(
        dimensions: Vec<usize>,
        mut value_builder: impl FnMut(Vec<usize>) -> Result<T, E>,
    ) -> Result<NVec<T>, E> {
        let mut data = Vec::new();
        for index in nd_index_iter(dimensions.clone()) {
            data.push(value_builder(index)?);
        }
        Ok(Self::from_vec_and_dims(
            data,
            dimensions.iter().map(|index| *index as usize).collect(),
        ))
    }

    pub fn build(
        dimensions: Vec<usize>,
        mut value_builder: impl FnMut(Vec<usize>) -> T,
    ) -> NVec<T> {
        let mut data = Vec::new();
        for index in nd_index_iter(dimensions.clone()) {
            data.push(value_builder(index));
        }
        Self::from_vec_and_dims(
            data,
            dimensions.iter().map(|index| *index as usize).collect(),
        )
    }

    pub fn new(dimensions: Vec<usize>, filler_value: T) -> NVec<T> {
        assert!(dimensions.len() > 0);
        let mut size = 1;
        for dim in dimensions.iter() {
            let dim = *dim;
            assert!(dim > 0);
            size *= dim;
        }
        let mut result = NVec {
            multipliers: Self::make_multipliers(&dimensions),
            dimensions,
            data: Vec::with_capacity(size),
        };
        for _ in 0..size {
            result.data.push(filler_value.clone());
        }
        result
    }

    pub fn from_vec(items: Vec<T>) -> NVec<T> {
        NVec {
            multipliers: vec![1],
            dimensions: vec![items.len()],
            data: items,
        }
    }

    pub fn from_vec_and_dims(items: Vec<T>, dimensions: Vec<usize>) -> NVec<T> {
        let mut size = 1;
        for dim in dimensions.iter() {
            size *= *dim;
        }
        assert!(items.len() == size);
        NVec {
            multipliers: Self::make_multipliers(&dimensions),
            dimensions: dimensions,
            data: items,
        }
    }

    pub fn collect(sub_arrays: Vec<NVec<T>>) -> NVec<T> {
        assert!(sub_arrays.len() > 0);

        let mut dimensions = sub_arrays[0].dimensions.clone();
        let mut multipliers = sub_arrays[0].multipliers.clone();
        for sub_array in sub_arrays.iter() {
            assert!(sub_array.dimensions == dimensions);
        }
        dimensions.insert(0, sub_arrays.len());
        multipliers.insert(0, sub_arrays[0].data.len());

        let mut data = Vec::new();
        for mut sub_array in sub_arrays.into_iter() {
            data.append(&mut sub_array.data);
        }

        NVec {
            multipliers,
            dimensions,
            data,
        }
    }

    pub fn map<F, U>(&self, mutator: F) -> NVec<U>
    where
        F: Fn(&T) -> U,
        U: Clone,
    {
        let mut new_items = Vec::new();
        for old_item in self.data.iter() {
            new_items.push(mutator(old_item));
        }
        NVec::from_vec_and_dims(new_items, self.dimensions.clone())
    }

    fn convert_to_raw_index(&self, coordinate: &[usize]) -> usize {
        let mut index = 0;
        for (coord, multiplier) in coordinate.iter().zip(self.multipliers.iter()) {
            index += coord * multiplier;
        }
        index
    }

    pub fn is_slice_inside(&self, coordinate: &[usize]) -> bool {
        if coordinate.len() > self.dimensions.len() {
            return false;
        }
        for (coord, max) in coordinate.iter().zip(self.dimensions.iter()) {
            if coord >= max {
                return false;
            }
        }
        true
    }

    pub fn is_inside(&self, coordinate: &[usize]) -> bool {
        if coordinate.len() != self.dimensions.len() {
            return false;
        }
        for (coord, max) in coordinate.iter().zip(self.dimensions.iter()) {
            if coord >= max {
                return false;
            }
        }
        true
    }

    pub fn set_item(&mut self, coordinate: &[usize], value: T) {
        assert!(self.is_inside(coordinate));
        let index = self.convert_to_raw_index(coordinate);
        self.data[index] = value;
    }

    pub fn clone_slice(&self, coordinate: &[usize]) -> Self {
        assert!(self.is_slice_inside(coordinate));
        let slice_order = coordinate.len();
        let new_dimensions = Vec::from(&self.dimensions[slice_order..]);
        let new_multipliers = Vec::from(&self.multipliers[slice_order..]);
        let start_index = self.convert_to_raw_index(coordinate);
        let size = if new_dimensions.len() == 0 {
            1
        } else {
            new_dimensions[0] * new_multipliers[0]
        };
        NVec {
            dimensions: new_dimensions,
            multipliers: new_multipliers,
            data: (&self.data[start_index..start_index + size]).into(),
        }
    }

    pub fn borrow_item(&self, coordinate: &[usize]) -> &T {
        assert!(self.is_inside(coordinate));
        &self.data[self.convert_to_raw_index(coordinate)]
    }

    pub fn borrow_item_mut(&mut self, coordinate: &[usize]) -> &mut T {
        assert!(self.is_inside(coordinate));
        let index = self.convert_to_raw_index(coordinate);
        &mut self.data[index]
    }

    pub fn borrow_all_items(&self) -> &Vec<T> {
        &self.data
    }

    pub fn borrow_dimensions(&self) -> &[usize] {
        &self.dimensions
    }
}

pub struct NdIndexIter {
    // Dimensions are stored in reverse to make calculations easier.
    dimensions: Vec<usize>,
    next_index: usize,
    total: usize,
}

impl Iterator for NdIndexIter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index == self.total {
            return None;
        }

        let mut result = Vec::with_capacity(self.dimensions.len());
        let mut counter = self.next_index;
        for dimension in &self.dimensions {
            result.push(counter % dimension);
            counter /= dimension;
        }
        result.reverse();
        self.next_index += 1;
        Some(result)
    }
}

pub fn nd_index_iter(mut dimensions: Vec<usize>) -> NdIndexIter {
    dimensions.reverse();
    let mut total = 1;
    for dimension in &dimensions {
        total *= dimension;
    }
    NdIndexIter {
        dimensions,
        next_index: 0,
        total,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn store_fetch_1d() {
        let mut array = NVec::new(vec![8], 0 as usize);
        for x in 0..8 {
            array.set_item(&vec![x], x * 2);
        }
        for x in 0..8 {
            assert!(*array.borrow_item(&vec![x]) == x * 2);
        }
    }

    #[test]
    fn store_fetch_2d() {
        // [4][3]usize
        let mut array = NVec::new(vec![4, 3], 0 as usize);
        for x in 0..4 {
            for y in 0..3 {
                array.set_item(&vec![x, y], x + y * 10);
            }
        }
        for x in 0..4 {
            for y in 0..3 {
                assert!(*array.borrow_item(&vec![x, y]) == x + y * 10);
            }
        }
    }

    #[test]
    fn collect_2d() {
        // 3x [6]usize
        let mut arrays = Vec::with_capacity(3);
        for x in 0..3 {
            let mut array = NVec::new(vec![6], 0 as usize);
            for y in 0..6 {
                array.set_item(&vec![y], x + y * 10);
            }
            arrays.push(array);
        }
        // [3][6]usize
        let collected_array = NVec::collect(arrays);
        for x in 0..3 {
            for y in 0..6 {
                assert!(*collected_array.borrow_item(&vec![x, y]) == x + y * 10);
            }
        }
    }

    #[test]
    fn slice_2d() {
        // [4][3]usize
        let mut array = NVec::new(vec![4, 3], 0 as usize);
        for x in 0..4 {
            for y in 0..3 {
                array.set_item(&vec![x, y], x + y * 10);
            }
        }
        for x in 0..4 {
            let slice = array.clone_slice(&vec![x]);
            for y in 0..3 {
                assert!(*slice.borrow_item(&vec![y]) == x + y * 10);
            }
        }
    }
}
