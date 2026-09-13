use super::frames::Animation;
use smithay::input::pointer::CursorIcon;
use std::{fs::File, io};
use xcursor::{CursorTheme, parser::parse_xcursor_stream};

/// Prefer aliases in the selected theme over canonical names in an ancestor.
pub(super) fn load(
    theme: &CursorTheme,
    icon: CursorIcon,
    target: u32,
    size: u32,
) -> io::Result<Animation> {
    let mut paths: Vec<_> = std::iter::once(icon.name())
        .chain(icon.alt_names().iter().copied())
        .filter_map(|name| theme.load_icon_with_depth(name))
        .collect();
    paths.sort_by_key(|(_, depth)| *depth);
    let mut error = io::Error::new(
        io::ErrorKind::NotFound,
        format!("cursor {} not found", icon.name()),
    );
    for (path, _) in paths {
        match File::open(path)
            .and_then(|mut file| parse_xcursor_stream(&mut file))
            .and_then(|images| Animation::new(images, target, size))
        {
            Ok(animation) => return Ok(animation),
            Err(failure) => error = failure,
        }
    }
    Err(error)
}
