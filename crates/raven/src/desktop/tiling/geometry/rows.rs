/// Resize a row against all rows on its selected side, distributing the change
/// by their available space above the minimum, as in smart master resizing.
pub(in crate::desktop::tiling) fn resize(rows: &mut [f64], row: usize, before: bool, delta: i32) {
    let affected = if before { 0..row } else { row + 1..rows.len() };
    if affected.is_empty() {
        return;
    }
    let minimum = (rows.iter().sum::<f64>() / rows.len() as f64 * 0.2)
        .floor()
        .max(1.0)
        .min(rows.iter().copied().fold(f64::INFINITY, f64::min));
    let room: f64 = affected.clone().map(|i| rows[i] - minimum).sum();
    let change = (if before {
        -(delta as f64)
    } else {
        delta as f64
    })
    .clamp(minimum - rows[row], room);
    rows[row] += change;
    let count = affected.len() as f64;
    for i in affected {
        let share = if room > 0.0 {
            (rows[i] - minimum) / room
        } else {
            1.0 / count
        };
        rows[i] -= change * share;
    }
}
