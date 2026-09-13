use crate::geometry::absolute;
use smithay::utils::Rectangle;

#[test]
fn normalized_absolute_motion_uses_output_origin_and_clamps_edges() {
    let bounds = Rectangle::new((-1920, 40).into(), (1920, 1080).into());
    assert_eq!(
        absolute(50, 25, 100, 100, bounds),
        Some((-960.0, 310.0).into())
    );
    assert_eq!(
        absolute(100, 100, 100, 100, bounds),
        Some((-1.0, 1119.0).into())
    );
    assert_eq!(
        absolute(200, 200, 100, 100, bounds),
        Some((-1.0, 1119.0).into())
    );
    assert_eq!(absolute(0, 0, 0, 100, bounds), None);
    assert_eq!(absolute(0, 0, 100, 0, bounds), None);
    assert_eq!(
        absolute(
            1,
            1,
            100,
            100,
            Rectangle::new((0, 0).into(), (0, 100).into())
        ),
        None
    );
}
