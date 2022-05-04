#![feature(test)]

extern crate nalgebra as na;
extern crate test;

use ark_bls12_381::Fr;
use ark_ff::PrimeField;
use ark_std::{rand::RngCore, UniformRand};
use na::{DMatrix, DVector};

fn frievalds<P: PrimeField, R: RngCore>(
    a: &DMatrix<P>,
    b: &DMatrix<P>,
    c: &DMatrix<P>,
    rng: &mut R,
) -> bool {
    let r = P::rand(rng);
    let x: DVector<P> = DVector::from_vec(
        (0..a.nrows())
            .scan(P::from(1u8), |r_m, _| {
                *r_m = *r_m * r;
                Some(*r_m)
            })
            .collect::<Vec<P>>(),
    );
    let y = c * &x;
    let z = a * (b * x);

    y == z
}

fn main() {
    let (nrows, ncols) = (10, 10);
    let mut rng = ark_std::test_rng();
    let a = DMatrix::from_fn(nrows, ncols, |_, _| Fr::rand(&mut rng));
    let b = DMatrix::from_fn(nrows, ncols, |_, _| Fr::rand(&mut rng));
    let c = &a * &b;

    match frievalds(&a, &b, &c, &mut rng) {
        true => println!("YES"),
        false => println!("NO"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_completeness() {
        // TODO: Define a field of small order and run the algorithm
        // N times, tallying the number of correct verifications to determine
        // the probability of 'YES'
    }

    #[test]
    fn test_soundness() {
        // TODO: Do as in the completeness test, but randomly perturb a single
        // element of matrix 'c'.
    }

    #[bench]
    fn bench_frievalds_runtime_50(bencher: &mut Bencher) {
        let (nrows, ncols) = (50, 50);
        let mut rng = ark_std::test_rng();
        let a = DMatrix::from_fn(nrows, ncols, |_, _| Fr::rand(&mut rng));
        let b = DMatrix::from_fn(nrows, ncols, |_, _| Fr::rand(&mut rng));
        let c = &a * &b;
        bencher.iter(|| frievalds(&a, &b, &c, &mut rng));
    }

    #[bench]
    fn bench_matmul_runtime_50(bencher: &mut Bencher) {
        let (nrows, ncols) = (50, 50);
        let mut rng = ark_std::test_rng();
        let a = DMatrix::from_fn(nrows, ncols, |_, _| Fr::rand(&mut rng));
        let b = DMatrix::from_fn(nrows, ncols, |_, _| Fr::rand(&mut rng));
        let c = &a * &b;
        bencher.iter(|| c == &a * &b);
    }
}
