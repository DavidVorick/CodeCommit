use crate::app_error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Model {
    #[default]
    Gemini2_5Pro,
    Gpt5,
}

impl Model {
    fn from_str(s: &str) -> Result<Self, AppError> {
        match s {
            "gemini-2.5-pro" => Ok(Model::Gemini2_5Pro),
            "gpt-5" => Ok(Model::Gpt5),
            _ => Err(AppError::Config(format!("Unsupported model: {s}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Workflow {
    #[default]
    CommitCode,
    ConsistencyCheck,
}

pub struct CliArgs {
    pub model: Model,
    pub workflow: Workflow,
}

pub fn parse_cli_args() -> Result<CliArgs, AppError> {
    let mut args = std::env::args().skip(1);
    let mut model = Model::default();
    let mut workflow: Option<Workflow> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--model" => {
                let model_str = args.next().ok_or_else(|| {
                    AppError::Config("Missing value for --model argument".to_string())
                })?;
                model = Model::from_str(&model_str)?;
            }
            "--consistency-check" | "--consistency" | "--cc" => {
                if workflow.is_some() {
                    return Err(AppError::Config(
                        "It is an error to trigger more than one workflow at a time.".to_string(),
                    ));
                }
                workflow = Some(Workflow::ConsistencyCheck);
            }
            _ => {
                return Err(AppError::Config(format!("Unknown argument: {arg}")));
            }
        }
    }

    Ok(CliArgs {
        model,
        workflow: workflow.unwrap_or_default(),
    })
}
