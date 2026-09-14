use smithay::utils::{Physical, Point, Rectangle, Size};
#[derive(Clone, Debug)]
pub(crate) struct Selection {
    pub size: Size<i32, Physical>,
    pub rect: Rectangle<i32, Physical>,
    anchor: Option<Point<i32, Physical>>,
    pub outlined: bool,
}
impl Selection {
    pub fn new(size: Size<i32, Physical>) -> Option<Self> {
        if size.w <= 0
            || size.h <= 0
            || i64::from(size.w) * i64::from(size.h) * 4 > 256 * 1024 * 1024
        {
            return None;
        }
        Some(Self {
            size,
            rect: Rectangle::from_size(size),
            anchor: None,
            outlined: false,
        })
    }
    fn point(&self, p: Point<f64, Physical>) -> Point<i32, Physical> {
        (
            p.x.floor().clamp(0.0, (self.size.w - 1) as f64) as i32,
            p.y.floor().clamp(0.0, (self.size.h - 1) as f64) as i32,
        )
            .into()
    }
    pub fn begin(&mut self, p: Point<f64, Physical>) {
        self.outlined = true;
        self.anchor = Some(self.point(p));
        self.motion(p);
    }
    pub fn motion(&mut self, p: Point<f64, Physical>) {
        let Some(a) = self.anchor else {
            return;
        };
        let b = self.point(p);
        self.rect = Rectangle::new(
            (a.x.min(b.x), a.y.min(b.y)).into(),
            ((a.x - b.x).abs() + 1, (a.y - b.y).abs() + 1).into(),
        );
    }
    pub fn end(&mut self, p: Point<f64, Physical>) {
        self.motion(p);
        self.anchor = None;
    }
    pub fn dragging(&self) -> bool {
        self.anchor.is_some()
    }
}
