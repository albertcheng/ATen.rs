use rayon::prelude::*;

impl Tensor {
    pub fn par_add(&self, other: &Tensor) -> Tensor {
        let result: Vec<f32> = self
            .data
            .iter()
            .zip(other.data.iter())
            .par_bridge()
            .map(|(a, b)| a + b)
            .collect();

        Tensor {
            data: ArrayD::from_shape_vec(self.data.raw_dim(), result).unwrap(),
        }
    }
}

