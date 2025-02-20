#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_addition() {
        let a = Tensor::new(&[2, 2]);
        let b = Tensor::new(&[2, 2]);
        let c = a.add(&b);
        assert_eq!(c.data.shape(), &[2, 2]);
    }
}

