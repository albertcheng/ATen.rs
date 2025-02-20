pub struct Tensor {
    data: ArrayD<f32>,
    grad: Option<ArrayD<f32>>,  // Store gradients
}

impl Tensor {
    pub fn backward(&mut self) {
        if self.grad.is_none() {
            self.grad = Some(ArrayD::ones(self.data.raw_dim())); // Initialize gradient
        }
    }
}

