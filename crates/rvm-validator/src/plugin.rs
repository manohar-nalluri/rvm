use rvm_types::diagnostic::ValidationReport;

/// Trait for custom validation plugins.
///
/// Implement this trait to add custom validation rules
/// (e.g., company-specific style checks).
pub trait ValidatorPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn validate(&self, tex_content: &str, report: &mut ValidationReport);
}

/// Registry for managing validation plugins.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn ValidatorPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, plugin: Box<dyn ValidatorPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn run_all(&self, tex_content: &str, report: &mut ValidationReport) {
        for plugin in &self.plugins {
            plugin.validate(tex_content, report);
        }
    }
}
