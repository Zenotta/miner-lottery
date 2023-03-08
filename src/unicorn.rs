//! Implementation of a UNICORN process, as per Lenstra and Wesolowski's "Random Zoo"
//! paper (https://eprint.iacr.org/2015/366.pdf). The eval-verify processes use modular
//! square root (sloth) with swapped neighbours, but this particular implementation may
//! need to be optimised in future given certain seed or modulus sizes.
//!
//! The goal of a UNICORN is to provide an uncontestable randomly generated number. The source
//! of the uncontestability is the seed, which is meant to be generated from multiple, random
//! oracle sources (eg. tweets). In the sloth implementation the seed is then run through a
//! function which is slow to compute but quick to verify (VDF, or Verifiable Delay Function)
//! and produces a witness value (for trapdoor verification) and the hash of the witness `g`.
//!
//! Although sloths have the extra ability to be slowed by a specific time length (through setting
//! the iterations, or `l`), any function that has slow evaluation and quick verification will
//! suffice for UNICORN needs.
//!
//! Given the seed and witness values, anybody is able to verify the authenticity of the number
//! generated.

use crate::utils::rug_integer;
use bincode::serialize;
use rug::integer::{IsPrime, Order};
use rug::Integer;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::error;

/// Number of rounds for Miller Rabin primality testing
pub const MR_PRIME_ITERS: u32 = 15;

/// Constructs the seed for a new lottery Unicorn
///
/// ### Arguments
///
/// * `tx_inputs` - Input transactions
/// * `participant_list` - List of miners participating in block round
/// * `last_winning_hashes` - The hashes of the winning PoWs from 2 blocks ago
pub fn construct_seed(
    public_key_inputs: &[String]
) -> Integer {
    // Transaction inputs (sOot)
    let pki = hex::encode(Sha256::digest(&serialize(public_key_inputs).unwrap()));
    Integer::from_str_radix(&pki, 16).unwrap()
}

/// Constructs the lottery Unicorn
///
/// ### Arguments
///
/// * `seed`         - Result of construct_seed
/// * `fixed_params` - UNICORN parameter to use
pub fn construct_unicorn(seed: Integer, fixed_params: &UnicornFixedParam) -> UnicornInfo {
    let unicorn = Unicorn {
        seed,
        modulus: Integer::from_str_radix(&fixed_params.modulus, 10).unwrap(),
        iterations: fixed_params.iterations,
        security_level: fixed_params.security,
    };

    let (w, g): (Integer, String) = match unicorn.eval() {
        Some((w, g)) => (w, g),
        None => panic!("UNICORN construction failed"),
    };

    UnicornInfo {
        unicorn,
        witness: w,
        g_value: g,
    }
}

/// Fixed parameters for unicorn
#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct UnicornFixedParam {
    /// UNICORN modulus number
    pub modulus: String,
    /// UNICORN iterations
    pub iterations: u64,
    /// UNICORN security level
    pub security: u32,
}

/// UNICORN-relevant info for use on a RAFT
#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct UnicornInfo {
    pub unicorn: Unicorn,
    pub g_value: String,
    #[serde(with = "rug_integer")]
    pub witness: Integer,
}

/// UNICORN struct, with the following fields:
///
/// - modulus (`p`)
/// - iterations (`l`)
/// - seed (`s`)
/// - witness (`w`)
/// - security_level (`k`)
#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Unicorn {
    pub iterations: u64,
    pub security_level: u32,
    #[serde(with = "rug_integer")]
    pub seed: Integer,
    #[serde(with = "rug_integer")]
    pub modulus: Integer,
}

impl Unicorn {
    /// Sets the seed for the UNICORN. Returns the commitment value `c`, as per
    /// Lenstra and Wesolowski recommendations
    ///
    /// ### Arguments
    ///
    /// * `seed`    - Seed to set
    pub fn set_seed(&mut self, seed: Integer) -> String {
        let u = hex::encode(Sha256::digest(&serialize(&seed.to_u64()).unwrap()));
        let c = hex::encode(Sha256::digest(u.as_bytes()));

        self.seed = seed;

        c
    }

    /// Evaluation of the Sloth VDF given internal params and a seed value,
    /// producing an uncontestable random number. Returns the raw witness value and hash `g`
    ///
    /// Mentioned in Section 3.3 of Lenstra et al's "Random Zoo", the modulus must be congruent
    /// to 3 % 4, so we can use this requirement to implement a slow modular square root through the
    /// exponent of `w`, the iterated value which will eventually become the witness.
    ///
    /// The general process as per Lenstra et al:
    /// - Let w0 be such that ̂w0 = seed (note that 0 ≤ w < 2^2k ≤ p).
    /// - For i = 1,2,...,l in succession let wi ← τ(wi−1).
    /// - Let g ← hash(wl) and w ← wl.
    /// - Return g and w as the output and quit.
    pub fn eval(&self) -> Option<(Integer, String)> {
        if !self.is_valid_modulus() {
            error!("Modulus for UNICORN eval invalid");
            return None;
        }

        let mut w = self.seed.clone().div_rem_floor(self.modulus.clone()).1;

        // The slow modular square root
        let exponent = (self.modulus.clone() + 1) / 4;

        for _ in 0..self.iterations {
            self.xor_for_overflow(&mut w);

            w.pow_mod_mut(&exponent, &self.modulus).unwrap();
        }

        let digits = w.to_digits::<u8>(Order::MsfBe);
        let g = hex::encode(digits);

        Some((w, g))
    }

    /// Verifies a particular unicorn given a witness value. This is the "trapdoor"
    /// function for public use. This process is quick in comparison to `eval`, as the
    /// process is a simple power raise with a modulo
    ///
    /// The general process as per Lenstra et al:
    /// - Replace w by (τ^−1)^l (w).
    /// - If w != int(u) then return “false” and quit.
    /// - Return “true” and quit.
    ///
    /// ### Arguments
    ///
    /// * `seed`    - Seed to verify
    /// * `witness` - Witness value for trapdoor verification
    pub fn verify(&self, seed: Integer, witness: Integer) -> bool {
        let square: Integer = 2u64.into();
        let mut w = witness;

        for _ in 0..self.iterations {
            // Fast squaring modulo
            w.pow_mod_mut(&square, &self.modulus).unwrap();

            let inv_w = -w;
            w = inv_w.div_rem_floor(self.modulus.clone()).1;
            self.xor_for_overflow(&mut w);
        }

        w == seed.div_rem_floor(self.modulus.clone()).1
    }

    /// Predicate for a valid modulus `p`
    ///
    /// As per Lenstra et al, requirements are as follows:
    /// - `p` must be large and prime
    /// - `p >= 2^2k` where `k` is a chosen security level
    fn is_valid_modulus(&self) -> bool {
        self.modulus >= 2u64.pow(2 * self.security_level)
            && !matches!(self.modulus.is_probably_prime(MR_PRIME_ITERS), IsPrime::No)
    }

    /// Performs a XOR of the input `x` as a basic secure permutation
    /// against modulus overflow
    ///
    /// ### Arguments
    ///
    /// * `w` - Input to XOR
    fn xor_for_overflow(&self, w: &mut Integer) {
        *w ^= 1;

        while *w >= self.modulus || *w == 0 {
            *w ^= 1;
        }
    }
}

/*---- TESTS ----*/

#[cfg(test)]
mod unicorn_tests {
    use super::*;

    const TEST_HASH: &str = "1eeb30c7163271850b6d018e8282093ac6755a771da6267edf6c9b4fce9242ba";
    const WITNESS: &str = "3519722601447054908751517254890810869415446534615259770378249754169022895693105944708707316137352415946228979178396400856098248558222287197711860247275230167";

    fn create_unicorn() -> Unicorn {
        let modulus_str: &str = "6864797660130609714981900799081393217269435300143305409394463459185543183397656052122559640661454554977296311391480858037121987999716643812574028291115057151";
        let modulus = Integer::from_str_radix(modulus_str, 10).unwrap();
        let seed = Integer::from_str_radix(TEST_HASH, 16).unwrap();

        Unicorn {
            modulus,
            iterations: 1_000,
            security_level: 1,
            seed,
        }
    }

    #[test]
    /// Checks that a valid unicorn can be evaluated
    fn eval_valid_unicorn() {
        let uni = create_unicorn();

        let eval = uni.eval().unwrap();

        assert_eq!(
            eval,
            (
                Integer::from_str_radix(WITNESS, 10).unwrap(),
                "0106834db40e90d1cafaa9e4c1981873186ebf019629852059aaf8e4ca35da01ca37041a4b475387dde0667c192ec18d1733d147ea9bfafa35ee4b05f74943e3d3d7".to_string()
            )
        );
    }

    #[test]
    /// Checks that an invalid modulus fail eval
    fn eval_invalid_modulus_unicorn() {
        let mut uni = create_unicorn();
        uni.modulus = Integer::from(2);

        let eval = uni.eval();

        assert_eq!(eval, None);
    }

    #[test]
    /// Checks that unicorn is succeed only with correct witness
    fn verify_unicorn() {
        let uni = create_unicorn();

        let good = uni.verify(
            Integer::from_str_radix(TEST_HASH, 16).unwrap(),
            Integer::from_str_radix(WITNESS, 10).unwrap(),
        );
        let bad = uni.verify(
            Integer::from_str_radix(TEST_HASH, 16).unwrap(),
            Integer::from(8),
        );

        assert_eq!((good, bad), (true, false));
    }
}
