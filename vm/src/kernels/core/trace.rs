use std::sync::Arc;

use afs_stark_backend::{
    config::{StarkGenericConfig, Val},
    rap::{get_air_name, AnyRap},
    Chip,
};
use p3_air::BaseAir;
use p3_field::PrimeField32;
use p3_matrix::dense::RowMajorMatrix;

use super::{columns::CoreCols, CoreChip};
use crate::arch::VmChip;

impl<F: PrimeField32> CoreChip<F> {
    /// Pad with NOP rows.
    pub fn pad_rows(&mut self) {
        let curr_height = self.rows.len();
        let correct_height = self.rows.len().next_power_of_two();
        for _ in 0..correct_height - curr_height {
            self.rows.push(self.make_blank_row().flatten());
        }
    }

    /// This must be called for each blank row and results should never be cloned; see [CoreCols::nop_row].
    fn make_blank_row(&self) -> CoreCols<F> {
        let pc = F::from_canonical_usize(self.state.pc);
        let timestamp = F::from_canonical_usize(self.state.timestamp);
        CoreCols::nop_row(self, pc, timestamp)
    }
}

impl<F: PrimeField32> VmChip<F> for CoreChip<F> {
    fn generate_trace(mut self) -> RowMajorMatrix<F> {
        self.pad_rows();

        RowMajorMatrix::new(self.rows.concat(), CoreCols::<F>::get_width(&self.air))
    }

    fn air_name(&self) -> String {
        get_air_name(&self.air)
    }

    fn generate_public_values(&mut self) -> Vec<F> {
        let first_row_pc = self.start_state.pc;
        let last_row_pc = self.state.pc;
        let mut result = vec![
            F::from_canonical_usize(first_row_pc),
            F::from_canonical_usize(last_row_pc),
        ];
        result.extend(self.public_values.iter().map(|pv| pv.unwrap_or(F::zero())));
        result
    }

    fn current_trace_height(&self) -> usize {
        self.rows.len()
    }

    fn trace_width(&self) -> usize {
        BaseAir::<F>::width(&self.air)
    }
}

impl<SC: StarkGenericConfig> Chip<SC> for CoreChip<Val<SC>>
where
    Val<SC>: PrimeField32,
{
    fn air(&self) -> Arc<dyn AnyRap<SC>> {
        Arc::new(self.air.clone())
    }
}