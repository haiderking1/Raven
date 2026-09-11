use std::error::Error;

pub(super) fn workspace(text: &str, name: &str) -> Result<String, Box<dyn Error>> {
    let suffix = format!(".name(\"{name}\")");
    for line in text.lines() {
        if let Some(end) = line.find(&suffix) {
            if let Some(start) = line[..end].find("ext_workspace_handle_v1") {
                return Ok(line[start..end].to_owned());
            }
        }
    }
    Err(format!("real Waybar did not receive workspace {name}").into())
}

pub(super) fn initialized(text: &str) -> bool {
    workspace(text, "1").is_ok()
        && workspace(text, "2").is_ok()
        && text.lines().any(|line| {
            line.contains("ext_workspace_group_handle_v1") && line.contains(".output_enter(")
        })
        && text.lines().any(|line| {
            line.contains("ext_workspace_group_handle_v1") && line.contains(".workspace_enter(")
        })
        && text
            .lines()
            .any(|line| line.contains("ext_workspace_manager_v1") && line.contains(".done()"))
}

/// The actual Waybar connection must request activation, receive the completed
/// state batch, then attach a new UI buffer. No synthetic workspace client runs.
pub(super) fn activated(text: &str, handle: &str, surface: u32) -> bool {
    let Some((_, after)) = text.split_once(&format!("{handle}.activate()")) else {
        return false;
    };
    let Some(commit) = after
        .lines()
        .find(|line| line.contains("ext_workspace_manager_v1") && line.contains(".commit()"))
    else {
        return false;
    };
    let Some((_, after)) = after.split_once(commit) else {
        return false;
    };
    let Some((_, after)) = after.split_once(&format!("{handle}.state(")) else {
        return false;
    };
    let Some(done) = after
        .lines()
        .find(|line| line.contains("ext_workspace_manager_v1") && line.contains(".done()"))
    else {
        return false;
    };
    let Some((_, after)) = after.split_once(done) else {
        return false;
    };
    let surface = format!("wl_surface#{surface}");
    let Some((_, after)) = after.split_once(&format!("{surface}.attach(wl_buffer")) else {
        return false;
    };
    after.contains(&format!("{surface}.commit()"))
}
