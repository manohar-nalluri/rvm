use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rvm_types::{RvmError, RvmResult};

/// A reusable LaTeX section template.
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub content: String,
    pub path: PathBuf,
}

/// Load all templates from the templates directory.
pub fn load_all(templates_dir: &Path) -> RvmResult<Vec<Template>> {
    let mut templates = Vec::new();

    if !templates_dir.exists() {
        return Ok(templates);
    }

    for entry in std::fs::read_dir(templates_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "tex") {
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let content = std::fs::read_to_string(&path)?;
            templates.push(Template {
                name,
                content,
                path,
            });
        }
    }

    Ok(templates)
}

/// Load a single template by name.
pub fn load(templates_dir: &Path, name: &str) -> RvmResult<Template> {
    let path = templates_dir.join(format!("{}.tex", name));
    if !path.exists() {
        return Err(RvmError::TemplateNotFound(name.to_string()));
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(Template {
        name: name.to_string(),
        content,
        path,
    })
}

/// Apply variable substitutions to a template.
pub fn render(template: &Template, variables: &HashMap<String, String>) -> String {
    let mut result = template.content.clone();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}
