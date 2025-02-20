use std::ops::Add;

impl Add for Tensor {
    type Output = Tensor;

    fn add(self, rhs: Tensor) -> Tensor {
        Tensor {
            data: self.data + rhs.data,
        }
    }
}

