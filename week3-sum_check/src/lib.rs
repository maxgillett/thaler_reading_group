use ark_ff::{fields::Fp64, Field, MontBackend, MontConfig, PrimeField};
use ark_poly::multivariate::{SparsePolynomial, SparseTerm, Term};
use ark_poly::{DenseMVPolynomial, Polynomial};
use std::marker::PhantomData;

struct Prover<F: PrimeField, P: Polynomial<F>> {
    poly: P,
    phantom: PhantomData<F>,
}

struct Verifier<F: PrimeField, P: Polynomial<F>> {
    poly: P,
    oracle: Oracle<F, P>,
    phantom: PhantomData<F>,
}

struct Oracle<F: PrimeField, P: Polynomial<F>> {
    poly: P,
    phantom: PhantomData<F>,
}

enum Message<F: PrimeField, P: Polynomial<F>> {
    PolynomialSubmission(P),
    PolynomialVerification(P),
    RandomElement(F),
}

struct Transcript<F: PrimeField, P: Polynomial<F>> {
    messages: Vec<Message<F, P>>,
}

impl<F, P> Prover<F, P>
where
    F: PrimeField,
    P: Polynomial<F>,
{
    fn new(poly: P) -> Self {
        Self {
            poly,
            phantom: PhantomData,
        }
    }
    fn send_C1(&self, ts: &mut Transcript<F, P>) {
        unimplemented!()
    }
    fn send_poly(&self, ts: &mut Transcript<F, P>) {
        unimplemented!()
    }
}

impl<F, P> Verifier<F, P>
where
    F: PrimeField,
    P: Polynomial<F>,
{
    fn new(poly: P, oracle: Oracle<F, P>) -> Self {
        Self {
            poly,
            oracle,
            phantom: PhantomData,
        }
    }
    fn verify_poly(&self, ts: &mut Transcript<F, P>) {
        unimplemented!()
    }
    /// Check that g_v(r_v) = g(r_1,...,r_v)
    fn verify_poly_v(&self, ts: &mut Transcript<F, P>) {
        //let r = ts.get_random_elements();
        //self.oracle.evaluate_at(r);
        unimplemented!()
    }
    fn draw_random_element(&self, ts: &mut Transcript<F, P>) {
        unimplemented!()
    }
}

impl<F, P> Oracle<F, P>
where
    F: PrimeField,
    P: Polynomial<F>,
{
    fn new(poly: P) -> Self {
        Self {
            poly,
            phantom: PhantomData,
        }
    }
    /// Evaluate the stored polynomial g at the point x
    fn evaluate_at(&self, x: P::Point) -> F {
        self.poly.evaluate(&x)
    }
}

impl<F: PrimeField, P: Polynomial<F>> Transcript<F, P> {
    fn new(num_rounds: usize) -> Self {
        Self { messages: vec![] }
    }

    fn get_random_elements(&self) -> P::Point {
        unimplemented!()
    }
}

#[derive(Debug)]
enum Error {}

/// Description of sum-check protocol rounds:
///
/// Round 1
/// - Prover sends C_1 (claimed to be the summed evaluation of g(x) over
///   x \in {0,1}^v), along  with the univariate polynomial g_1(X_1).
/// - Verifier checks that C_1 = g_1(0) + g_1(1), and chooses a random
///   element r_1 uniformly from F_p.
///
/// Round j (1<j<v)
/// - Prover sends g_j (claimed to be the polynomial g(r_1,..,r_j-1,X_j,x_{j+1},..,x_v)
///   summed over {x_{j+1},..,x_v}, where v is the number of rounds.
/// - Verifier checks that deg(g_j) <= deg_j(g), that g_{j-1}(r_{j-1}) = g_j(0) + g_j(1),
///   and chooses a random element r_j uniformly from F_p.
///
/// Round v
/// - Prover sends g_v (claimed to be the polynomial g(r_1,..,r_v-1,x_v).
/// - Verifier checks that g_{v-1}(r_{v-1}) = g_v(0) + g_v(1), that deg(g_v) <= deg_v(g),
///   and chooses a random element r_v uniformly from F_p. Verifier queries the Oracle
///   to check that g_v(r_v) = g(r_1,...,r_v).
///
fn sumcheck<F: PrimeField>(
    num_rounds: usize,
    poly: SparsePolynomial<F, SparseTerm>,
) -> Result<bool, Error> {
    let prover = Prover::new(poly.clone());
    let oracle = Oracle::new(poly.clone());
    let verifier = Verifier::new(poly, oracle);

    let mut ts: Transcript<F, SparsePolynomial<F, SparseTerm>> = Transcript::new(num_rounds);

    for i in 0..num_rounds - 1 {
        prover.send_poly(&mut ts);
        verifier.verify_poly(&mut ts);
        verifier.draw_random_element(&mut ts);
    }
    verifier.verify_poly_v(&mut ts);

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(MontConfig)]
    #[modulus = "5"]
    #[generator = "3"]
    pub struct FqConfig;
    pub type Fr = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn test_example_execution() {
        // g(X_1, X_2, X_3) = 2X_1^3 + X_1X_3 + X_2X_3
        let poly: SparsePolynomial<Fr, SparseTerm> = SparsePolynomial::from_coefficients_slice(
            3,
            &[
                ("1".parse().unwrap(), SparseTerm::new(vec![(1, 1), (2, 1)])),
                ("1".parse().unwrap(), SparseTerm::new(vec![(0, 1), (2, 1)])),
                ("2".parse().unwrap(), SparseTerm::new(vec![(0, 3)])),
            ],
        );
        assert_eq!(sumcheck(3, poly).unwrap(), true)
    }
}
