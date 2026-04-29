//! 工作区路径相关能力。

use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// 生成工作区文件的绝对路径。
///
/// `relative_path` 由上层保证是项目内相对路径，这里只做路径拼接。
pub fn workspace_file_absolute_path(workspace_root: &Path, relative_path: &str) -> PathBuf {
    workspace_root.join(relative_path)
}

/// 推断当前进程所在目录，作为工作区根目录的默认值。
///
/// 获取失败时回退 `.`，保证启动流程不被环境异常阻断。
pub fn detect_workspace_root() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 规范化工作区目录：优先返回规范路径，失败时回退到原路径。
pub fn normalize_workspace_root(path: impl Into<PathBuf>) -> PathBuf {
    let path = path.into();
    fs::canonicalize(&path).unwrap_or(path)
}

/// 从工作区根目录推断项目名称。
///
/// 无法推断时回退为 `"workspace"`，用于 UI 标题等可展示字段。
pub fn project_name_from_root(workspace_root: &Path) -> String {
    workspace_root
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "workspace".into())
}

/// 从相对路径提取文件名，作为标签标题等展示用途。
pub fn file_name_from_path(relative_path: &str) -> String {
    Path::new(relative_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| relative_path.to_string())
}

/// 从文件路径推断语言名称（用于工具栏与标签元信息）。
///
/// 先看扩展名，再处理特殊文件名（如 `Dockerfile` / `Makefile`）。
pub fn language_from_path(relative_path: &str) -> String {
    let path = Path::new(relative_path);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_ascii_lowercase());

    match extension.as_deref() {
        Some("rs") => "Rust",
        Some("py") => "Python",
        Some("ts") | Some("mts") | Some("cts") => "TypeScript",
        Some("tsx") => "TypeScript React",
        Some("js") | Some("mjs") | Some("cjs") => "JavaScript",
        Some("jsx") => "JavaScript React",
        Some("go") => "Go",
        Some("java") => "Java",
        Some("kt") | Some("kts") => "Kotlin",
        Some("swift") => "Swift",
        Some("c") => "C",
        Some("h") => "C Header",
        Some("cc") | Some("cpp") | Some("cxx") | Some("hh") | Some("hpp") | Some("hxx") => "C++",
        Some("cs") => "C#",
        Some("php") => "PHP",
        Some("rb") => "Ruby",
        Some("sh") | Some("bash") | Some("zsh") => "Shell",
        Some("toml") => "TOML",
        Some("json") => "JSON",
        Some("yaml") | Some("yml") => "YAML",
        Some("xml") => "XML",
        Some("html") | Some("htm") => "HTML",
        Some("css") => "CSS",
        Some("scss") => "SCSS",
        Some("sql") => "SQL",
        Some("md") | Some("mdx") => "Markdown",
        Some("txt") => "Text",
        _ => match file_name.as_deref() {
            Some("dockerfile") => "Dockerfile",
            Some("makefile") => "Makefile",
            _ => "Unknown",
        },
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::language_from_path;

    #[test]
    /// 计算路径结果。
    fn language_from_path_uses_extension_mapping() {
        assert_eq!(language_from_path("src/main.rs"), "Rust");
        assert_eq!(language_from_path("web/app.tsx"), "TypeScript React");
        assert_eq!(language_from_path("scripts/build.zsh"), "Shell");
    }

    #[test]
    /// 计算路径文本结果。
    fn language_from_path_falls_back_to_plain_text() {
        assert_eq!(language_from_path("notes/README"), "Unknown");
    }

    #[test]
    /// 计算路径文件结果。
    fn language_from_path_supports_special_file_names() {
        assert_eq!(language_from_path("Dockerfile"), "Dockerfile");
        assert_eq!(language_from_path("build/Makefile"), "Makefile");
    }
}
