use crate::app_error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Model {
    #[default]
    Gemini3Pro,
    Gemini2_5Pro,
    Gpt5,
}

impl Model {
    fn from_str(s: &str) -> Result<Self, AppError> {
        match s {
            "gemini-3-pro" => Ok(Model::Gemini3Pro),
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
    Rollup,
}

#[derive(Debug, PartialEq, Default)]
pub struct CliArgs {
    pub model: Model,
    pub workflow: Workflow,
    pub force: bool,
    pub light_roll: bool,
}

pub fn parse_cli_args() -> Result<CliArgs, AppError> {
    parse_args(std::env::args().skip(1))
}

pub(crate) fn parse_args<T: Iterator<Item = String>>(mut args: T) -> Result<CliArgs, AppError> {
    let mut model = Model::default();
    let mut workflow: Option<Workflow> = None;
    let mut force = false;
    let mut light_roll = false;

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
            "--rollup" => {
                if workflow.is_some() {
                    return Err(AppError::Config(
                        "It is an error to trigger more than one workflow at a time.".to_string(),
                    ));
                }
                workflow = Some(Workflow::Rollup);
            }
            "--light-roll" | "--lr" => {
                light_roll = true;
            }
            "--force" | "--f" => {
                force = true;
            }
            _ => {
                return Err(AppError::Config(format!("Unknown argument: {arg}")));
            }
        }
    }

    let final_workflow = workflow.unwrap_or_default();

    if force && final_workflow != Workflow::CommitCode {
        return Err(AppError::Config(
            "The --force or --f flag can only be used with the 'committing-code' workflow."
                .to_string(),
        ));
    }

    if light_roll && final_workflow != Workflow::Rollup {
        return Err(AppError::Config(
            "The --light-roll flag can only be used with the --rollup workflow.".to_string(),
        ));
    }

    Ok(CliArgs {
        model,
        workflow: final_workflow,
        force,
        light_roll,
    })
}
