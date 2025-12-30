# API Signatures

pub enum Model {
    Gemini3Pro,
    Gemini2_5Pro,
    Gpt5,
}

pub enum Workflow {
    CommitCode,
    ConsistencyCheck,
    Rollup,
    Auto,
    Init(String),
}

pub struct CliArgs {
    pub model: Model,
    pub workflow: Workflow,
    pub force: bool,
    pub rollup_full: bool,
}

pub fn parse_cli_args() -> Result<CliArgs, AppError>

pub(crate) fn parse_args<T: Iterator<Item = String>>(mut args: T) -> Result<CliArgs, AppError>