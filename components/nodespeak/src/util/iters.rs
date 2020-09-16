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
