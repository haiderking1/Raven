use crate::desktop::appearance::InnerGaps;
use smithay::utils::{Logical, Rectangle};

/// A half-width master on the left, with the remaining windows stacked right.
/// Integer remainders belong to the right column and its first rows.
pub(super) fn spaced_master_stack(
    area: Rectangle<i32, Logical>,
    count: usize,
    gaps: InnerGaps,
) -> Vec<Rectangle<i32, Logical>> {
    if area.size.w <= 0 || area.size.h <= 0 {
        return Vec::new();
    }
    (0..count)
        .filter_map(|index| spaced_tile_at(area, count, index, gaps))
        .collect()
}

/// Rendering and input clipping need one allocation, not a newly allocated
/// vector containing every tile for every window being examined.
pub(super) fn spaced_tile_at(
    area: Rectangle<i32, Logical>,
    count: usize,
    index: usize,
    gaps: InnerGaps,
) -> Option<Rectangle<i32, Logical>> {
    if index >= count || area.size.w <= 0 || area.size.h <= 0 {
        return None;
    }
    // Positive overlapping allocations preserve XDG's nonzero-size contract
    // when the output is too small to partition into independent tiles.
    if count == 1 || area.size.w < 2 || count - 1 > area.size.h as usize {
        return Some(area);
    }
    let horizontal = gaps.horizontal.min(area.size.w - 2);
    let master_width = (area.size.w - horizontal) / 2;
    if index == 0 {
        return Some(Rectangle::new(area.loc, (master_width, area.size.h).into()));
    }
    let rows = (count - 1) as i32;
    let row = (index - 1) as i32;
    let vertical = if rows > 1 {
        gaps.vertical.min((area.size.h - rows) / (rows - 1))
    } else {
        0
    };
    let available = area.size.h - vertical * (rows - 1);
    let height = available / rows;
    let remainder = available % rows;
    let y = area.loc.y + row * (height + vertical) + row.min(remainder);
    Some(Rectangle::new(
        (area.loc.x + master_width + horizontal, y).into(),
        (
            area.size.w - master_width - horizontal,
            height + i32::from(row < remainder),
        )
            .into(),
    ))
}

#[cfg(test)]
fn master_stack(area: Rectangle<i32, Logical>, count: usize) -> Vec<Rectangle<i32, Logical>> {
    spaced_master_stack(area, count, InnerGaps::default())
}
#[cfg(test)]
mod tests;
