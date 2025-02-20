use ndarray::{ArrayD, IxDyn};

impl Tensor {
    pub fn add(&self, other: &Tensor) -> Tensor {
        Tensor {
            data: &self.data + &other.data,  // Element-wise addition
        }
    }

    pub fn mul(&self, other: &Tensor) -> Tensor {
        Tensor {
            data: &self.data * &other.data,  // Element-wise multiplication
        }
    }
}

