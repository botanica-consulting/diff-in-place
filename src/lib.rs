#![no_std]

pub trait DiffInPlace<T, const N: usize>
where
    T: PartialEq,
{
    /// Fallible version of `diff_in_place` for propagating errors.
    /// Perform an in-place diff between two const-size arrays, invoking the given function
    /// for each run of different elements, with the index into the array and
    /// the slice of different elements from the other array.
    ///
    ///
    /// # Arguments
    /// * `other`   - The other array to compare against.
    /// * `func`    - The function to call for each run of different elements.
    ///
    /// # Example
    /// ```
    ///     use diff_in_place::DiffInPlace;
    ///     let a = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    ///     let mut b = [0, 0, 1, 2, 0, 0, 0, 3, 4, 5];
    ///
    ///     a.try_diff_in_place(&b, |idx, diff| -> Result<(), ()> {
    ///         // println!("{}: {:?}", idx, diff);
    ///         // Prints:
    ///         // 2: [1, 2]
    ///         // 7: [3, 4, 5]
    ///         Ok(())
    ///     }).unwrap();
    /// ```
    fn try_diff_in_place<F, R>(&self, other: &[T; N], func: F) -> Result<(), R>
    where
        F: FnMut(usize, &[T]) -> Result<(), R>;

    /// Perform an in-place diff between two const-size arrays, invoking the given function
    /// for each run of different elements, with the index into the array and
    /// the slice of different elements from the other array.
    ///
    /// # Arguments
    /// * `other`   - The other array to compare against.
    /// * `func`    - The function to call for each run of different elements.
    ///
    /// # Example
    /// ```
    ///     use diff_in_place::DiffInPlace;
    ///     let a = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    ///     let mut b = [0, 0, 1, 2, 0, 0, 0, 3, 4, 5];
    ///
    ///     a.diff_in_place(&b, |idx, diff| {
    ///         // println!("{}: {:?}", idx, diff);
    ///         // Prints:
    ///         // 2: [1, 2]
    ///         // 7: [3, 4, 5]
    ///     });
    /// ```
    fn diff_in_place<F>(&self, other: &[T; N], mut func: F)
    where
        F: FnMut(usize, &[T]),
    {
        self.try_diff_in_place(other, |idx, diff| -> Result<(), ()> {
            func(idx, diff);
            Ok(())
        })
        .unwrap();
    }
}

#[derive(Copy, Clone)]
enum DiffState {
    Same,
    Different(usize),
}

impl<T, const N: usize> DiffInPlace<T, N> for [T; N]
where
    T: PartialEq + Copy,
{
    fn try_diff_in_place<F, R>(&self, other: &[T; N], mut func: F) -> Result<(), R>
    where
        F: FnMut(usize, &[T]) -> Result<(), R>,
    {
        // Go over the bytes in both arrays, comparing them.
        // For all different runs within the two buffers, call the diff function
        // with the index into the buffers and the bytes that need to be changed.
        let byte_for_byte = self.iter().zip(other.iter());
        let mut run_state = DiffState::Same;
        for (current, (left, right)) in byte_for_byte.enumerate() {
            match (run_state, left == right) {
                (DiffState::Same, false) => {
                    // We are starting an unequal run, preserve the current index
                    run_state = DiffState::Different(current);
                }
                (DiffState::Different(run_start), true) => {
                    // We are ending an unequal run, call the diff function
                    func(run_start, &other[run_start..current])?;
                    run_state = DiffState::Same;
                }
                _ => {
                    // Run state is unchanged
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fully_same() {
        let a = [0u8; 40];
        let b = [0u8; 40];
        a.diff_in_place(&b, |_, _| panic!("Should not be called"))
    }

    #[test]
    fn test_fully_different() {
        let a = [0u8; 40];
        let b = [1u8; 40];
        a.diff_in_place(&b, |idx, diff| {
            assert_eq!(idx, 0);
            assert_eq!(diff, &[1u8]);
        });
    }

    #[test]
    fn test_start_different() {
        let a = [0u8; 40];
        let mut b = [0u8; 40];

        b[..10].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        const EXPECTED_CALLS: [(usize, &[u8]); 1] = [(0usize, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];

        a.diff_in_place(&b, |idx, diff| {
            let (expected_idx, expected_diff) = EXPECTED_CALLS[0];
            assert_eq!(idx, expected_idx);
            assert_eq!(diff, expected_diff);
        });
    }

    #[test]
    fn test_middle_different() {
        let a = [0u8; 40];
        let mut b = [0u8; 40];

        b[10..20].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        const EXPECTED_CALLS: [(usize, &[u8]); 1] = [(10usize, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];

        a.diff_in_place(&b, |idx, diff| {
            let (expected_idx, expected_diff) = EXPECTED_CALLS[0];
            assert_eq!(idx, expected_idx);
            assert_eq!(diff, expected_diff);
        })
    }

    #[test]
    fn test_end_different() {
        let a = [0u8; 40];
        let mut b = [0u8; 40];

        b[30..].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        const EXPECTED_CALLS: [(usize, &[u8]); 1] = [(30usize, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];

        a.diff_in_place(&b, |idx, diff| {
            let (expected_idx, expected_diff) = EXPECTED_CALLS[0];
            assert_eq!(idx, expected_idx);
            assert_eq!(diff, expected_diff);
        })
    }

    #[test]
    fn test_multiple_different() {
        let a = [0u8; 40];
        let mut b = [0u8; 40];

        b[..10].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        b[20..25].copy_from_slice(&[11, 12, 13, 14, 15]);
        b[39] = 20;

        const EXPECTED_CALLS: [(usize, &[u8]); 3] = [
            (0usize, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            (20usize, &[11, 12, 13, 14, 15]),
            (39usize, &[20]),
        ];

        let mut call_idx = 0;
        a.diff_in_place(&b, |idx, diff| {
            let (expected_idx, expected_diff) = EXPECTED_CALLS[call_idx];
            assert_eq!(idx, expected_idx);
            assert_eq!(diff, expected_diff);
            call_idx += 1;
        })
    }

    #[test]
    fn test_other_types() {
        let a = [0.0f32; 40];
        let mut b = [0.0f32; 40];

        b[..10].copy_from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7., 8., 9., 10.]);

        const EXPECTED_CALLS: [(usize, &[f32]); 1] =
            [(0usize, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7., 8., 9., 10.])];

        a.diff_in_place(&b, |idx, diff| {
            let (expected_idx, expected_diff) = EXPECTED_CALLS[0];
            assert_eq!(idx, expected_idx);
            assert_eq!(diff, expected_diff);
        });
    }

    #[test]
    #[should_panic]
    fn test_fallible_diff() {
        let a = [0u8; 40];
        let mut b = [0u8; 40];

        b[..10].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        a.try_diff_in_place(&b, |_idx, _diff| -> Result<(), ()> { Err(()) })
            .unwrap();
    }
}
