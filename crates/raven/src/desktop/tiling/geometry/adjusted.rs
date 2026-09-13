use super::spaced_master_stack;
use crate::desktop::appearance::InnerGaps;
use smithay::utils::{Logical, Rectangle};

#[derive(Clone, Debug)]
pub(in crate::desktop::tiling) struct Splits {
    pub master: f64,
    pub rows: Vec<f64>,
}
impl Default for Splits {
    fn default() -> Self {
        Self {
            master: 0.5,
            rows: Vec::new(),
        }
    }
}

pub(in crate::desktop::tiling) fn frames(
    area: Rectangle<i32, Logical>,
    count: usize,
    gaps: InnerGaps,
    splits: &Splits,
) -> Vec<Rectangle<i32, Logical>> {
    let mut frames = spaced_master_stack(area, count, gaps);
    if count < 2 || area.size.w < 2 || count - 1 > area.size.h as usize {
        return frames;
    }
    let horizontal = frames[1].loc.x - frames[0].loc.x - frames[0].size.w;
    let available = area.size.w - horizontal;
    let master = ((available as f64 * splits.master).floor() as i32).clamp(1, available - 1);
    frames[0].size.w = master;
    for frame in &mut frames[1..] {
        frame.loc.x = area.loc.x + master + horizontal;
        frame.size.w = available - master;
    }
    let rows = count - 1;
    if splits.rows.len() != rows || rows < 2 {
        return frames;
    }
    let vertical = frames[2].loc.y - frames[1].loc.y - frames[1].size.h;
    let available = area.size.h - vertical * (rows as i32 - 1);
    let sum: f64 = splits.rows.iter().sum();
    let mut cumulative = 0.0;
    let mut previous = 0;
    for (row, frame) in frames[1..].iter_mut().enumerate() {
        cumulative += splits.rows[row];
        let end = if row + 1 == rows {
            available
        } else {
            ((available as f64 * cumulative / sum).round() as i32)
                .clamp(previous + 1, available - (rows - row - 1) as i32)
        };
        frame.loc.y = area.loc.y + previous + row as i32 * vertical;
        frame.size.h = end - previous;
        previous = end;
    }
    frames
}
