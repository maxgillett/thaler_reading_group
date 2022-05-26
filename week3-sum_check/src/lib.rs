use ark_ff::{fields::Fp64, Field, MontBackend, MontConfig, PrimeField};
use ark_poly::multivariate::{SparsePolynomial, SparseTerm, Term};
use ark_poly::{DenseMVPolynomial, Polynomial};
use rand::rngs::StdRng;
use std::marker::PhantomData;

struct Prover<F: PrimeField, P: Polynomial<F>> {
    poly: P,
    phantom: PhantomData<F>,
}

struct Verifier<F: PrimeField, P: Polynomial<F>> {
    poly: P,
    oracle: Oracle<F, P>,
    rng: StdRng,
    phantom: PhantomData<F>,
}

struct Oracle<F: PrimeField, P: Polynomial<F>> {
    poly: P,
    phantom: PhantomData<F>,
}

enum Message<F: PrimeField, P: Polynomial<F>> {
    ProverSendPolynomial(P),
    VerifierSendRandomElement(F),
}

struct Round<F: PrimeField, P: Polynomial<F>> {
    poly: P,
    r: F,
}

struct Transcript<F: PrimeField, P: Polynomial<F>> {
    num_rounds: usize,
    messages: Vec<Message<F, P>>,
    rounds: Vec<Round<F, P>>,
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
        // TODO: Sum over indeterminates
        unimplemented!()
    }
    fn send_poly(&self, idx: usize, ts: &mut Transcript<F, P>) {
        // TODO: Substitute random elements in polynomial and sum over indeterminates
        // let r_vec = (0..idx).map(|i| ts.get_random_element(i)).collect::<Vec<_>>();
        unimplemented!()
    }
}

impl<F, P> Verifier<F, P>
where
    F: PrimeField,
    P: Polynomial<F, Point = Vec<F>>,
{
    fn new(poly: P, oracle: Oracle<F, P>) -> Self {
        Self {
            poly,
            oracle,
            rng: ark_std::test_rng(),
            phantom: PhantomData,
        }
    }
    /// Check that g_{j-1}(r_{j-1}) = g_j(0) + g_j(1)
    fn verify_poly(&self, round: usize, ts: &mut Transcript<F, P>) {
        let g_j = ts.get_poly(round);
        let g_j_1 = ts.get_poly(round - 1);
        let r_j_1 = ts.get_random_element(round - 1);
        assert_eq!(
            g_j.evaluate(&[F::from(0u8)].into()) + g_j.evaluate(&[F::from(1u8)].into()),
            g_j_1.evaluate(&vec![r_j_1])
        );
    }
    /// Check that g_v(r_v) = g(r_1,...,r_v)
    fn verify_poly_v(&self, ts: &mut Transcript<F, P>) {
        let v = ts.num_rounds - 1;
        let g_v = ts.get_poly(v);
        let r_vec = (0..ts.rounds.len())
            .map(|i| ts.get_random_element(i))
            .collect::<Vec<_>>();
        let g_eval = self.oracle.evaluate_at(&r_vec);
        let g_v_eval = g_v.evaluate(&[r_vec[v]].into());
        assert_eq!(g_eval, g_v_eval,);
    }
    /// Draw a random element uniformly from F and append to transcript
    fn draw_random_element(&mut self, round: usize, ts: &mut Transcript<F, P>) {
        let r = F::rand(&mut self.rng);
        ts.append(round, Message::VerifierSendRandomElement(r));
    }
}

impl<F, P> Oracle<F, P>
where
    F: PrimeField,
    P: Polynomial<F, Point = Vec<F>>,
{
    fn new(poly: P) -> Self {
        Self {
            poly,
            phantom: PhantomData,
        }
    }
    /// Evaluate the stored polynomial g at the point x
    fn evaluate_at(&self, x: &[F]) -> F {
        self.poly.evaluate(&x.into())
    }
}

impl<F, P> Transcript<F, P>
where
    F: PrimeField,
    P: Polynomial<F>,
{
    fn new(num_rounds: usize) -> Self {
        Self {
            num_rounds,
            messages: Vec::with_capacity(num_rounds * 2),
            rounds: Vec::with_capacity(num_rounds),
        }
    }

    fn append(&mut self, idx: usize, msg: Message<F, P>) {
        match &msg {
            Message::ProverSendPolynomial(p) => {
                self.rounds[idx].poly = p.clone();
            }
            Message::VerifierSendRandomElement(r) => {
                self.rounds[idx].r = *r;
            }
            _ => {}
        }
        self.messages.push(msg);
    }

    fn get_poly(&self, idx: usize) -> &P {
        &self.rounds[idx].poly
    }

    fn get_random_element(&self, idx: usize) -> F {
        self.rounds[idx].r
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
    let mut verifier = Verifier::new(poly, oracle);

    let mut ts: Transcript<F, SparsePolynomial<F, SparseTerm>> = Transcript::new(num_rounds);

    for i in 0..num_rounds - 1 {
        prover.send_poly(i, &mut ts);
        verifier.verify_poly(i, &mut ts);
        verifier.draw_random_element(i, &mut ts);
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
