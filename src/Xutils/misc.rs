pub fn clear_vec<Val>(mut to_delete: Vec<usize>, vals: &mut Vec<Val>) {
    to_delete.sort_unstable();
    to_delete.reverse();
    for index in to_delete {
        vals.remove(index);
    }
}
