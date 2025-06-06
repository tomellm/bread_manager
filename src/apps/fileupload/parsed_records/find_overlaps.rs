use std::sync::Arc;

use hermes::container::data::ImplData;
use itertools::Itertools;
use lazy_async_promise::ImmediateValuePromise;

use crate::{
    apps::fileupload::{
        parsed_records::results_with_overlaps::{
            ImportOverlap, ImportResultWithOverlap,
        },
        FileToParse,
    },
    model::data_import::{row::ImportRow, DataImport},
};

use super::ParsedRecords;

pub fn find_overlaps(view: &mut ParsedRecords, file: FileToParse) {
    let FileToParse {
        file,
        profile: Some(profile),
        ..
    } = file
    else {
        unreachable!(
                "Please select a profile for the file [{}] since no profile was selected.",
                file.file.name
            );
    };

    let all_imports = Arc::clone(view.imports.data());

    let future = ImmediateValuePromise::new(async move {
        let file = file.path.unwrap();

        let file_str = std::fs::read_to_string(&file).unwrap();
        let import_rows = file_str
            .lines()
            .enumerate()
            .map(|(index, line)| ImportRow::init(line.to_string(), index))
            .collect_vec();

        let new_import = DataImport::init(profile.uuid, &file_str, file);
        assert!(!import_rows.is_empty());

        let overlaps = all_imports
            .iter()
            .filter_map(|import| {
                let mut first_match = None;
                let sorted_counts = import.rows.iter().counts_by(|row| {
                    for (index, new_row) in import_rows.iter().enumerate() {
                        if !profile.is_margin(index, import_rows.len())
                            && row.row_content.eq(&new_row.row_content)
                        {
                            if first_match.is_none() {
                                first_match = Some(index);
                            }
                            return true;
                        }
                    }
                    false
                });

                match sorted_counts.get(&true).unwrap_or(&0) {
                    0 => None,
                    _ => {
                        let mut import = import.clone();
                        import.sort_by_index();
                        Some(ImportOverlap::new(import, first_match.unwrap()))
                    }
                }
            })
            .collect_vec();

        Ok(ImportResultWithOverlap::new(
            new_import,
            import_rows,
            profile,
            overlaps,
        ))
    });
    view.import_state.set_overlaps(future);
}
