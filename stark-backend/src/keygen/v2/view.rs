use itertools::Itertools;
use p3_challenger::FieldChallenger;
use p3_uni_stark::StarkGenericConfig;

use crate::{
    config::Com,
    keygen::v2::types::{
        MultiStarkProvingKeyV2, MultiStarkVerifyingKeyV2, StarkProvingKeyV2, StarkVerifyingKeyV2,
    },
};

pub(crate) struct MultiStarkVerifyingKeyV2View<'a, SC: StarkGenericConfig> {
    pub per_air: Vec<&'a StarkVerifyingKeyV2<SC>>,
}

pub(crate) struct MultiStarkProvingKeyV2View<'a, SC: StarkGenericConfig> {
    pub air_ids: Vec<usize>,
    pub per_air: Vec<&'a StarkProvingKeyV2<SC>>,
}

impl<SC: StarkGenericConfig> MultiStarkVerifyingKeyV2<SC> {
    /// Returns a view with all airs.
    pub(crate) fn full_view(&self) -> MultiStarkVerifyingKeyV2View<SC> {
        self.view(&(0..self.per_air.len()).collect_vec())
    }
    pub(crate) fn view(&self, air_ids: &[usize]) -> MultiStarkVerifyingKeyV2View<SC> {
        MultiStarkVerifyingKeyV2View {
            per_air: air_ids.iter().map(|&id| &self.per_air[id]).collect(),
        }
    }
}
impl<SC: StarkGenericConfig> MultiStarkProvingKeyV2<SC> {
    pub(crate) fn view(&self, air_ids: Vec<usize>) -> MultiStarkProvingKeyV2View<SC> {
        let per_air = air_ids.iter().map(|&id| &self.per_air[id]).collect();
        MultiStarkProvingKeyV2View { air_ids, per_air }
    }
}

impl<'a, SC: StarkGenericConfig> MultiStarkVerifyingKeyV2View<'a, SC> {
    /// Returns the preprocessed commit of each AIR. If the AIR does not have a preprocessed trace, returns None.
    pub fn preprocessed_commits(&self) -> Vec<Option<Com<SC>>> {
        self.per_air
            .iter()
            .map(|vk| {
                vk.preprocessed_data
                    .as_ref()
                    .map(|data| data.commit.clone())
            })
            .collect()
    }
    /// Returns all non-empty preprocessed commits.
    pub fn flattened_preprocessed_commits(&self) -> Vec<Com<SC>> {
        self.preprocessed_commits().into_iter().flatten().collect()
    }
    /// Returns challenges of each phase.
    pub fn sample_challenges(&self, challenger: &mut SC::Challenger) -> Vec<Vec<SC::Challenge>> {
        // Generate 2 permutation challenges
        let num_challenges_to_sample = self.num_challenges_to_sample();
        assert!(num_challenges_to_sample.len() <= 1);
        num_challenges_to_sample
            .iter()
            .map(|&num_challenges| {
                (0..num_challenges)
                    .map(|_| challenger.sample_ext_element::<SC::Challenge>())
                    .collect_vec()
            })
            .collect()
    }
    pub fn num_phases(&self) -> usize {
        self.per_air
            .iter()
            .map(|vk| {
                // Consistency check
                let num = vk.params.width.after_challenge.len();
                assert_eq!(num, vk.params.num_challenges_to_sample.len());
                assert_eq!(num, vk.params.num_exposed_values_after_challenge.len());
                num
            })
            .max()
            .unwrap_or(0)
    }
    pub fn num_challenges_to_sample(&self) -> Vec<usize> {
        let num_phases = self.num_phases();
        (0..num_phases)
            .map(|phase_idx| {
                self.per_air
                    .iter()
                    .map(|vk| {
                        *vk.params
                            .num_challenges_to_sample
                            .get(phase_idx)
                            .unwrap_or(&0)
                    })
                    .max()
                    .unwrap_or_else(|| panic!("No challenges used in challenge phase {phase_idx}"))
            })
            .collect()
    }
}

impl<'a, SC: StarkGenericConfig> MultiStarkProvingKeyV2View<'a, SC> {
    pub fn vk_view(&self) -> MultiStarkVerifyingKeyV2View<SC> {
        MultiStarkVerifyingKeyV2View {
            per_air: self.per_air.iter().map(|pk| &pk.vk).collect(),
        }
    }
}