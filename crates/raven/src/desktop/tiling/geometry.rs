use smithay::utils::{Logical, Rectangle};

/// A half-width master on the left, with the remaining windows stacked right.
/// Integer remainders belong to the right column and its first rows.
pub(super) fn master_stack(
    area: Rectangle<i32, Logical>,
    count: usize,
) -> Vec<Rectangle<i32, Logical>> {
    if area.size.w <= 0 || area.size.h <= 0 {
        return Vec::new();
    }
    (0..count)
        .filter_map(|index| tile_at(area, count, index))
        .collect()
}

/// Rendering and input clipping need one allocation, not a newly allocated
/// vector containing every tile for every window being examined.
pub(super) fn tile_at(
    area: Rectangle<i32, Logical>,
    count: usize,
    index: usize,
) -> Option<Rectangle<i32, Logical>> {
    if index >= count || area.size.w <= 0 || area.size.h <= 0 {
        return None;
    }
    // Positive overlapping allocations preserve XDG's nonzero-size contract
    // when the output is too small to partition into independent tiles.
    if count == 1 || area.size.w < 2 || count - 1 > area.size.h as usize {
        return Some(area);
    }
    let master_width = area.size.w / 2;
    if index == 0 {
        return Some(Rectangle::new(area.loc, (master_width, area.size.h).into()));
    }
    let rows = (count - 1) as i32;
    let row = (index - 1) as i32;
    let height = area.size.h / rows;
    let remainder = area.size.h % rows;
    let y = area.loc.y + row * height + row.min(remainder);
    Some(Rectangle::new(
        (area.loc.x + master_width, y).into(),
        (
            area.size.w - master_width,
            height + i32::from(row < remainder),
        )
            .into(),
    ))
}

#[cfg(test)]
mod tests;
