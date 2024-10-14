use afs_compiler::ir::Config;
use afs_stark_backend::keygen::types::{MultiStarkVerifyingKey, StarkVerifyingKey};
use ax_sdk::config::baby_bear_poseidon2_outer::BabyBearPoseidon2OuterConfig;
use p3_baby_bear::BabyBear;
use p3_bn254_fr::{Bn254Fr, DiffusionMatrixBN254};
use p3_challenger::MultiField32Challenger;
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_fri::{
    BatchOpening, CommitPhaseProofStep, FriProof, QueryProof, TwoAdicFriPcs, TwoAdicFriPcsProof,
};
use p3_merkle_tree::FieldMerkleTreeMmcs;
use p3_poseidon2::{Poseidon2, Poseidon2ExternalMatrixGeneral};
use p3_symmetric::{MultiField32PaddingFreeSponge, TruncatedPermutation};

use crate::{
    digest::DigestVal,
    types::{
        MultiStarkVerificationAdvice, StarkVerificationAdvice,
        VerifierSinglePreprocessedDataInProgram,
    },
};

const WIDTH: usize = 3;
const RATE: usize = 16;
const DIGEST_WIDTH: usize = 1;

#[derive(Clone, Default, Debug)]
pub struct OuterConfig;

impl Config for OuterConfig {
    type N = Bn254Fr;
    type F = BabyBear;
    type EF = BinomialExtensionField<BabyBear, 4>;
}

/// A configuration for outer recursion.
pub type OuterVal = BabyBear;
pub type OuterChallenge = BinomialExtensionField<OuterVal, 4>;
pub type OuterPerm =
    Poseidon2<Bn254Fr, Poseidon2ExternalMatrixGeneral, DiffusionMatrixBN254, WIDTH, 5>;
pub type OuterHash =
    MultiField32PaddingFreeSponge<OuterVal, Bn254Fr, OuterPerm, WIDTH, RATE, DIGEST_WIDTH>;
pub type OuterDigest = [Bn254Fr; 1];
pub type OuterCompress = TruncatedPermutation<OuterPerm, 2, 1, WIDTH>;
pub type OuterValMmcs = FieldMerkleTreeMmcs<BabyBear, Bn254Fr, OuterHash, OuterCompress, 1>;
pub type OuterChallengeMmcs = ExtensionMmcs<OuterVal, OuterChallenge, OuterValMmcs>;
pub type OuterDft = Radix2DitParallel;
pub type OuterChallenger = MultiField32Challenger<OuterVal, Bn254Fr, OuterPerm, WIDTH>;
pub type OuterPcs = TwoAdicFriPcs<OuterVal, OuterDft, OuterValMmcs, OuterChallengeMmcs>;

pub type OuterQueryProof = QueryProof<OuterChallenge, OuterChallengeMmcs>;
pub type OuterCommitPhaseStep = CommitPhaseProofStep<OuterChallenge, OuterChallengeMmcs>;
pub type OuterFriProof = FriProof<OuterChallenge, OuterChallengeMmcs, OuterVal>;
pub type OuterBatchOpening = BatchOpening<OuterVal, OuterValMmcs>;
pub type OuterPcsProof =
    TwoAdicFriPcsProof<OuterVal, OuterChallenge, OuterValMmcs, OuterChallengeMmcs>;

pub(crate) fn new_from_outer_vkv2(
    vk: StarkVerifyingKey<BabyBearPoseidon2OuterConfig>,
) -> StarkVerificationAdvice<OuterConfig> {
    let StarkVerifyingKey {
        preprocessed_data,
        params,
        quotient_degree,
        symbolic_constraints,
    } = vk;
    StarkVerificationAdvice {
        preprocessed_data: preprocessed_data.map(|data| {
            let commit: [Bn254Fr; DIGEST_WIDTH] = data.commit.into();
            VerifierSinglePreprocessedDataInProgram {
                commit: DigestVal::N(commit.to_vec()),
            }
        }),
        width: params.width,
        quotient_degree,
        num_public_values: params.num_public_values,
        num_challenges_to_sample: params.num_challenges_to_sample,
        num_exposed_values_after_challenge: params.num_exposed_values_after_challenge,
        symbolic_constraints: symbolic_constraints.constraints,
    }
}

/// Create MultiStarkVerificationAdvice for the outer config.
pub fn new_from_outer_multi_vk(
    vk: &MultiStarkVerifyingKey<BabyBearPoseidon2OuterConfig>,
) -> MultiStarkVerificationAdvice<OuterConfig> {
    let num_challenges_to_sample = vk.num_challenges_to_sample();
    let MultiStarkVerifyingKey::<BabyBearPoseidon2OuterConfig> { per_air } = vk;
    MultiStarkVerificationAdvice {
        per_air: per_air
            .clone()
            .into_iter()
            .map(new_from_outer_vkv2)
            .collect(),
        num_challenges_to_sample,
    }
}