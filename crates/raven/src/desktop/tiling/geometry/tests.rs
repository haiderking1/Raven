use super::master_stack;
use smithay::utils::{Logical, Rectangle};

#[test]
fn master_stack_covers_odd_sized_output_without_rounding_gaps() {
    let area: Rectangle<i32, Logical> = Rectangle::new((13, 17).into(), (1001, 601).into());
    assert!(master_stack(area, 0).is_empty());
    assert_eq!(master_stack(area, 1), vec![area]);
    assert_eq!(
        master_stack(area, 4),
        vec![
            Rectangle::new((13, 17).into(), (500, 601).into()),
            Rectangle::new((513, 17).into(), (501, 201).into()),
            Rectangle::new((513, 218).into(), (501, 200).into()),
            Rectangle::new((513, 418).into(), (501, 200).into()),
        ]
    );
    let tiny = Rectangle::new((0, 0).into(), (1, 1).into());
    assert_eq!(master_stack(tiny, 2), vec![tiny; 2]);
}
