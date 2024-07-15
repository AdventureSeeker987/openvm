use std::{collections::HashSet, sync::Arc};

use p3_field::{AbstractField, PrimeField, PrimeField64};
use p3_matrix::dense::RowMajorMatrix;
use p3_uni_stark::{StarkGenericConfig, Val};

use crate::{
    common::page::Page, is_less_than_tuple::columns::IsLessThanTupleCols,
    multitier_page_rw_checker::leaf_page_air::PageRwAir, range_gate::RangeCheckerGateChip,
    sub_chip::LocalTraceInstructions,
};

use super::LeafPageAir;

impl<const COMMITMENT_LEN: usize> LeafPageAir<COMMITMENT_LEN> {
    // The trace is the whole page (including the is_alloc column)
    pub fn generate_cached_trace<F: PrimeField64>(&self, page: Page) -> RowMajorMatrix<F> {
        page.gen_trace()
    }

    pub fn generate_main_trace<SC: StarkGenericConfig>(
        &self,
        page: &Page,
        commit: Vec<u32>,
        range: (Vec<u32>, Vec<u32>),
        range_checker: Arc<RangeCheckerGateChip>,
        internal_indices: &HashSet<Vec<u32>>,
    ) -> RowMajorMatrix<Val<SC>>
    where
        Val<SC>: PrimeField64 + PrimeField,
    {
        assert!(commit.len() == COMMITMENT_LEN);
        let mut final_page_aux_rows = match &self.page_chip {
            PageRwAir::Final(fin) => {
                fin.gen_aux_trace::<SC>(page, range_checker.clone(), internal_indices)
            }
            _ => RowMajorMatrix::new(vec![], 1),
        };
        RowMajorMatrix::new(
            page.iter()
                .enumerate()
                .flat_map(|(i, row)| {
                    let mut trace_row = vec![];
                    trace_row.extend(commit.clone());
                    trace_row.push(self.air_id);
                    if !self.is_init {
                        trace_row.extend(range.0.clone());
                        trace_row.extend(range.1.clone());
                        trace_row.extend(vec![0; 2]);
                        let mut trace_row: Vec<Val<SC>> = trace_row
                            .into_iter()
                            .map(Val::<SC>::from_canonical_u32)
                            .collect();
                        {
                            let tuple: IsLessThanTupleCols<Val<SC>> = self
                                .is_less_than_tuple_air
                                .clone()
                                .unwrap()
                                .idx_start
                                .generate_trace_row((
                                    row.idx.to_vec(),
                                    range.0.clone(),
                                    range_checker.clone(),
                                ));
                            let aux = tuple.aux;
                            let io = tuple.io;
                            trace_row[COMMITMENT_LEN + 2 * range.0.len() + 1] = io.tuple_less_than;
                            trace_row.extend(aux.flatten());
                        }
                        {
                            let tuple: IsLessThanTupleCols<Val<SC>> = self
                                .is_less_than_tuple_air
                                .clone()
                                .unwrap()
                                .end_idx
                                .generate_trace_row((
                                    range.1.clone(),
                                    row.idx.to_vec(),
                                    range_checker.clone(),
                                ));
                            let aux = tuple.aux;
                            let io = tuple.io;
                            trace_row[COMMITMENT_LEN + 2 * range.0.len() + 2] = io.tuple_less_than;
                            trace_row.extend(aux.flatten());
                        }
                        {
                            trace_row.append(&mut final_page_aux_rows.row_mut(i).to_vec());
                        }
                        trace_row
                    } else {
                        trace_row
                            .into_iter()
                            .map(Val::<SC>::from_wrapped_u32)
                            .collect::<Vec<Val<SC>>>()
                    }
                })
                .collect(),
            self.air_width() - (1 + self.idx_len + self.data_len),
        )
    }
}