use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Clone, Debug)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub desktop_file: PathBuf,
    pub icon: Option<String>,
}

fn truthy(value: Option<&String>) -> bool {
    value.is_some_and(|v| v.eq_ignore_ascii_case("true"))
}

fn command_exists(command: &str) -> bool {
    let path = Path::new(command);
    if command.contains('/') {
        return path.is_file();
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(command).is_file())
}

fn parse_desktop_file(path: &Path) -> Option<AppEntry> {
    let contents = fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let mut values = BTreeMap::<String, String>::new();
    for raw in contents.lines() {
        let line = raw.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if !key.contains('[') {
                values.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
    }
    if values.get("Type").is_some_and(|v| v != "Application")
        || truthy(values.get("Hidden"))
        || truthy(values.get("NoDisplay"))
    {
        return None;
    }
    let name = values.get("Name")?.trim();
    let exec = values.get("Exec")?.trim();
    if name.is_empty() || exec.is_empty() {
        return None;
    }
    if let Some(try_exec) = values.get("TryExec").filter(|v| !v.is_empty()) {
        if !command_exists(try_exec) {
            return None;
        }
    }
    Some(AppEntry {
        name: name.into(),
        exec: exec.into(),
        desktop_file: path.into(),
        icon: values.get("Icon").cloned(),
    })
}

fn visit(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "desktop") {
            out.push(path);
        }
    }
}

pub fn discover_apps() -> Vec<AppEntry> {
    let mut roots = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];
    if let Some(home) = std::env::var_os("XDG_DATA_HOME") {
        roots.push(PathBuf::from(home).join("applications"));
    } else if let Some(home) = dirs::home_dir() {
        roots.push(home.join(".local/share/applications"));
    }
    let mut paths = Vec::new();
    for root in roots {
        visit(&root, &mut paths);
    }
    paths.sort();
    paths.dedup();
    let mut apps: Vec<_> = paths.iter().filter_map(|p| parse_desktop_file(p)).collect();
    apps.sort_by_key(|app| app.name.to_lowercase());
    apps
}

fn expand(token: &str, app: &AppEntry) -> Option<String> {
    let mut result = String::new();
    let mut chars = token.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            result.push(ch);
            continue;
        }
        match chars.next()? {
            '%' => result.push('%'),
            'c' => result.push_str(&app.name),
            'k' => result.push_str(&app.desktop_file.to_string_lossy()),
            // File/URL placeholders are omitted until file-open actions exist.
            'f' | 'F' | 'u' | 'U' | 'd' | 'D' | 'n' | 'N' | 'i' | 'm' | 'v' => return None,
            _ => return None,
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

pub fn launch_app(app: &AppEntry) -> Result<(), String> {
    let tokens = shell_words::split(&app.exec).map_err(|e| format!("Exec invalide : {e}"))?;
    let args: Vec<String> = tokens.iter().filter_map(|t| expand(t, app)).collect();
    let Some(program) = args.first() else {
        return Err("Commande vide".into());
    };
    if !command_exists(program) {
        return Err(format!("Programme introuvable : {program}"));
    }
    Command::new(program)
        .args(&args[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Échec du lancement : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn ignores_hidden_desktop_entries() {
        let dir = std::env::temp_dir().join(format!(
            "wanderdesk-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let visible = dir.join("visible.desktop");
        let hidden = dir.join("hidden.desktop");
        fs::write(
            &visible,
            "[Desktop Entry]\nType=Application\nName=Example\nExec=/bin/true\n",
        )
        .unwrap();
        fs::write(
            &hidden,
            "[Desktop Entry]\nName=Hidden\nExec=/bin/true\nHidden=true\n",
        )
        .unwrap();
        assert_eq!(parse_desktop_file(&visible).unwrap().name, "Example");
        assert!(parse_desktop_file(&hidden).is_none());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn expands_name_and_drops_file_placeholders() {
        let app = AppEntry {
            name: "Test App".into(),
            exec: String::new(),
            desktop_file: "/tmp/test.desktop".into(),
            icon: None,
        };
        assert_eq!(
            expand("--title=%c", &app).as_deref(),
            Some("--title=Test App")
        );
        assert_eq!(expand("%U", &app), None);
        assert_eq!(expand("100%%", &app).as_deref(), Some("100%"));
    }
}
