use smithay::utils::{Logical, Rectangle};

/// A half-width master on the left, with the remaining windows stacked right.
/// Integer remainders belong to the right column and its first rows.
pub(super) fn master_stack(
    area: Rectangle<i32, Logical>,
    count: usize,
) -> Vec<Rectangle<i32, Logical>> {
    if count == 0 || area.size.w <= 0 || area.size.h <= 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![area];
    }
    // A tile must have positive dimensions: zero means "client chooses" in XDG.
    // On an output too small for this split, use overlapping full-output tiles
    // instead, preserving the focused window at the top of the stack.
    if area.size.w < 2 || count - 1 > area.size.h as usize {
        return vec![area; count];
    }
    let master_width = area.size.w / 2;
    let rows = (count - 1) as i32;
    let height = area.size.h / rows;
    let remainder = area.size.h % rows;
    let mut tiles = Vec::with_capacity(count);
    tiles.push(Rectangle::new(area.loc, (master_width, area.size.h).into()));
    let mut y = area.loc.y;
    for row in 0..rows {
        let row_height = height + i32::from(row < remainder);
        tiles.push(Rectangle::new(
            (area.loc.x + master_width, y).into(),
            (area.size.w - master_width, row_height).into(),
        ));
        y += row_height;
    }
    tiles
}

#[cfg(test)]
mod tests;
