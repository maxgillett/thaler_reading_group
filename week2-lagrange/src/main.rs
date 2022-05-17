use ark_ff::{fields::Fp64, MontBackend, MontConfig, PrimeField};
use itertools::Itertools;

#[derive(MontConfig)]
#[modulus = "5"]
#[generator = "3"]
pub struct FqConfig;
pub type Fr = Fp64<MontBackend<FqConfig, 1>>;

// Naive Langrage interpolation implementation (Lemma 3.7)
fn lagrange<P: PrimeField, F: Fn(&Vec<bool>) -> P>(f: F, n: usize) -> impl Fn(Vec<P>) -> P {
    let one = P::from(1u8);
    let mut f_eval: Vec<P> = vec![];
    let mut w_vec: Vec<Vec<bool>> = vec![];
    for w in (0..n).map(|_| [true, false]).multi_cartesian_product() {
        f_eval.push(f(&w));
        w_vec.push(w.clone());
    }

    move |x: Vec<P>| {
        let chi_eval = w_vec.iter().map(|w| {
            w.into_iter()
                .zip(&x)
                .map(|(w_i, x_i)| if *w_i { *x_i } else { one - *x_i })
                .product1::<P>()
        });

        f_eval
            .iter()
            .zip(chi_eval)
            .map(|(a, b)| *a * b.unwrap())
            .sum1::<P>()
            .unwrap()
    }
}

// TODO: Efficient Lagrange interpolation algorithm (Lemma 3.8)
#[allow(dead_code)]
fn lagrange_efficient() {}

// Function f mapping {0,1}^2 to prime field F_5 (see Figure 3.1)
fn f<P: PrimeField>(x: &Vec<bool>) -> P {
    assert_eq!(x.len(), 2, "invalid input domain size");
    P::from(match (x[0] as u8, x[1] as u8) {
        (0, 0) => 1,
        (0, 1) => 2,
        (1, 0) => 1,
        (1, 1) => 4,
        _ => panic!(),
    } as u32)
}

fn main() {
    let f_tilde = lagrange(f, 2);
    let res = f_tilde(vec![Fr::from(1u8), Fr::from(1u8)]);
    println!("{:?}", res);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_f() {
        for x in (0..2).map(|_| (0..5)).multi_cartesian_product() {
            let a = Fr::from(match (x[0], x[1]) {
                (0, 0) => 1,
                (0, 1) => 2,
                (0, 2) => 3,
                (0, 3) => 4,
                (0, 4) => 5,

                (1, 0) => 1,
                (1, 1) => 4,
                (1, 2) => 2,
                (1, 3) => 0,
                (1, 4) => 3,

                (2, 0) => 1,
                (2, 1) => 1,
                (2, 2) => 1,
                (2, 3) => 1,
                (2, 4) => 1,

                (3, 0) => 1,
                (3, 1) => 3,
                (3, 2) => 0,
                (3, 3) => 2,
                (3, 4) => 4,

                (4, 0) => 1,
                (4, 1) => 0,
                (4, 2) => 4,
                (4, 3) => 3,
                (4, 4) => 2,
                _ => panic!(),
            } as u32);
            let f_tilde = lagrange(f, 2);
            let b = f_tilde(vec![Fr::from(x[0]), Fr::from(x[1])]);
            assert_eq!(
                a, b,
                "incorrect evaluation of multilinear extension f_tilde"
            );
        }
    }
}
