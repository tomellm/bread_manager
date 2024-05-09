pub fn clear_vec<Val>(mut to_delete: Vec<usize>, vals: &mut Vec<Val>) {
    to_delete.sort();
    to_delete.reverse();
    to_delete.into_iter().for_each(|index| {
        vals.remove(index);
    });
    
}
