use ndarray::{ArrayD, IxDyn};

pub struct Tensor {
    data: ArrayD<f32>,  // Multi-dimensional array
}

impl Tensor {
    pub fn new(shape: &[usize]) -> Self {
        Tensor {
            data: ArrayD::<f32>::zeros(IxDyn(shape)),
        }
    }
}

