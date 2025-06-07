pub mod button_future;
pub mod expense_records;
pub mod option_display;
pub(crate) mod origins;
pub mod pagination;
pub mod soft_button;
pub mod table;
pub mod tags;

fn clamp_str(str: &str, max_len: usize) -> &str {
    match str.len() <= max_len {
        true => str,
        false => &str[0..max_len],
    }
}
