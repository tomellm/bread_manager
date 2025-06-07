use crate::model::{
    data_import::{row::ImportRow, DataImport},
    profiles::Profile,
};
use egui::Ui;

pub struct ImportResultWithOverlap {
    pub import: DataImport,
    pub rows: Vec<(RowSelectionStatus, ImportRow)>,
    pub profile: Profile,
    pub overlaps: Vec<ImportOverlap>,
}

impl ImportResultWithOverlap {
    pub fn new(
        import: DataImport,
        rows: Vec<ImportRow>,
        profile: Profile,
        overlaps: Vec<ImportOverlap>,
    ) -> Self {
        let rows_len = rows.len();
        Self {
            import,
            rows: rows
                .into_iter()
                .enumerate()
                .map(|(index, rec)| {
                    let state = RowSelectionStatus::from_in_margins(
                        profile.is_margin(index, rows_len),
                    );
                    (state, rec)
                })
                .collect(),
            profile,
            overlaps,
        }
    }

    pub fn remove_overlap(&mut self, index: usize) {
        self.overlaps.remove(index);
    }

    pub fn is_overlap_cleared(&self) -> bool {
        self.overlaps.is_empty()
    }
}

#[derive(Debug)]
pub struct RowSelectionStatus {
    pub include: Option<bool>,
}

impl RowSelectionStatus {
    pub fn from_in_margins(in_margins: bool) -> Self {
        match in_margins {
            true => Self { include: None },
            false => Self {
                include: Some(true),
            },
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut bool> {
        self.include.as_mut()
    }

    pub fn set(&mut self, val: bool) {
        if let Some(old_val) = self.include.as_mut() {
            *old_val = val;
        }
    }

    pub fn is_to_parse(&self) -> bool {
        self.include.unwrap_or_default()
    }
}

pub struct ImportOverlap {
    pub import: DataImport,
    pub first_match: usize,
}

impl ImportOverlap {
    pub fn new(import: DataImport, first_match: usize) -> Self {
        Self {
            import,
            first_match,
        }
    }
}

pub fn overlap_control_buttons(
    check_overlapping: &mut bool,
    uncheck_all: &mut bool,
    resolve_this_overlap: &mut bool,
    ui: &mut Ui,
) -> bool {
    let mut clear_parse = false;
    ui.horizontal(|ui| {
        *check_overlapping = ui.button("check all overlapping").clicked();
        *uncheck_all = ui.button("uncheck all").clicked();
        *resolve_this_overlap = ui
            .button("resolve this overlap")
            .on_hover_text(ACCEPT_THIS_OVERLAP_TEXT)
            .clicked();
        if ui
            .button("nevermind")
            .on_hover_text(NEVERMIND_OVERLAP_TEXT)
            .clicked()
        {
            clear_parse = true;
        }
    });
    clear_parse
}

const ACCEPT_THIS_OVERLAP_TEXT: &str = r#"
When you click this button you confirm that you have compared the left and right
side enough and checked the right rows and can move on to the next one or move
on to parsing the selected rows.
"#;

const NEVERMIND_OVERLAP_TEXT: &str = r#"
Nevermind I already imported this File. If all of the rows on the left and right
side overlap, click this to abort the whole parsing action.
"#;
