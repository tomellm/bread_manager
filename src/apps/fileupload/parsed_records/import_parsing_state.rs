use crate::{
    model::{
        data_import::DataImport, group::Group, profiles::ParseResult,
        transactions::Transaction,
    },
    utils::PromiseUtilities,
};
use itertools::Itertools;
use lazy_async_promise::ImmediateValuePromise;
use std::mem;

use super::ImportResultWithOverlap;

pub enum ImportParsingState {
    None,
    FindingOverlaps(ImmediateValuePromise<ImportResultWithOverlap>),
    OverlapsFound(ImportResultWithOverlap),
    Parsing(ImmediateValuePromise<(Vec<Transaction>, DataImport, Vec<Group>)>),
    Finished(Vec<Transaction>, DataImport, Vec<Group>),
}

impl ImportParsingState {
    pub fn ready_for_new(&self) -> bool {
        matches!(self, ImportParsingState::None)
    }
    pub fn set_overlaps(
        &mut self,
        future: ImmediateValuePromise<ImportResultWithOverlap>,
    ) {
        assert!(matches!(self, Self::None));
        let _ = mem::replace(self, ImportParsingState::FindingOverlaps(future));
    }

    pub fn start_parse(&mut self) {
        let ImportParsingState::OverlapsFound(overlaps) =
            mem::replace(self, ImportParsingState::None)
        else {
            unreachable!()
        };
        let future = async move {
            let ImportResultWithOverlap {
                mut import,
                profile,
                mut rows,
                ..
            } = overlaps;

            let to_parse_rows = rows
                .extract_if(.., |t| t.0.is_to_parse())
                .map(|t| t.1)
                .collect_vec();
            import.rows.extend(rows.into_iter().map(|t| t.1));

            let ParseResult {
                rows,
                groups,
                parsed_rows,
            } = profile.parse_file(to_parse_rows).unwrap();

            import.rows.extend(parsed_rows);
            (rows, import, groups)
        };

        let _ = mem::replace(self, ImportParsingState::Parsing(future.into()));
    }
    
    pub fn try_resolve(&mut self) {
        if let Self::FindingOverlaps(finding) = self {
            finding
                .poll_and_check_finished()
                .then(|| finding.take_expect())
        } else {
            None
        }
        .map(|value| mem::replace(self, Self::OverlapsFound(value)));

        if let Self::Parsing(parsing) = self {
            parsing
                .poll_and_check_finished()
                .then(|| parsing.take_expect())
        } else {
            None
        }
        .map(|value| {
            mem::replace(self, Self::Finished(value.0, value.1, value.2))
        });
    }
    pub fn clear(&mut self) {
        let _ = mem::replace(self, ImportParsingState::None);
    }
}
