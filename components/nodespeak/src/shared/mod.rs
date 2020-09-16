// TODO: Move to a more specific file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProxyMode {
    /// Use this as a regular array index.
    Keep,
    /// Throw the index away.
    Discard,
    /// Replace with index 0/
    Collapse,
}

impl ProxyMode {
    pub fn symbol(&self) -> &str {
        match self {
            Self::Keep => "",
            Self::Discard => ">X",
            Self::Collapse => ">1",
        }
    }
}

pub fn apply_proxy_to_index(proxy: &[(usize, ProxyMode)], index: &[usize]) -> Vec<usize> {
    let mut current_dimension = 0;
    let mut result = Vec::new();
    for proxy_dimension in proxy {
        match proxy_dimension.1 {
            ProxyMode::Keep => result.push(index[current_dimension]),
            ProxyMode::Discard => (),
            ProxyMode::Collapse => result.push(0),
        }
        current_dimension += 1;
    }
    result
}

pub struct NDIndexIter {
    position: usize,
    reverse_dimensions: Vec<usize>,
}

impl Iterator for NDIndexIter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut reverse_coord = Vec::new();
        let mut current_part = self.position;
        for dim in &self.reverse_dimensions {
            reverse_coord.push(current_part % dim);
            current_part /= dim;
        }
        if current_part == 0 {
            self.position += 1;
            reverse_coord.reverse();
            Some(reverse_coord)
        } else {
            None
        }
    }
}

impl NDIndexIter {
    pub fn new(mut dimensions: Vec<usize>) -> NDIndexIter {
        dimensions.reverse();
        NDIndexIter {
            position: 0,
            reverse_dimensions: dimensions,
        }
    }
}
