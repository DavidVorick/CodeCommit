# API Signatures

```rust
#[derive(Debug)]
pub struct Config {
    pub model: Model,
    pub api_key: String,
    pub query: String,
    pub system_prompts: String,
}

impl Config {
    pub fn load(args: &CliArgs) -> Result<Self, AppError>;
    pub(crate) fn get_query_from_editor() -> Result<String, AppError>;
    pub fn load_from_dir(args: &CliArgs, base_dir: &Path, query: String) -> Result<Self, AppError>;
}
```
