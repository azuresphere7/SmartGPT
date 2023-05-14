use std::{sync::{Mutex, Arc}, error::Error};

use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{ScriptValue, ProgramInfo, Command, CommandContext, Expression, GPTRunError};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    pub tool: String,
    pub args: ScriptValue
}

pub async fn run_command(
    out: &mut String,
    name: String, command: Command, 
    context: &mut CommandContext, args: ScriptValue
) -> Result<ScriptValue, Box<dyn Error>> {
    let result = command.run.invoke(context, args.clone()).await?;
    let args: Expression = args.clone().into();

    let json = serde_yaml::to_string(&result)
        .map_err(|_| GPTRunError("Could not parse ScriptValue as JSON.".to_string()))?;

    let text = format!("Tool use {name} {:?} returned:\n{}", args, json);
    out.push_str(&text);

    Ok(result)
}

pub fn run_action_sync(context: &mut CommandContext, action: Action) -> Result<String, Box<dyn Error>> {
    let command = context.plugins.iter()
        .flat_map(|el| &el.commands)
        .find(|el| el.name == action.tool)
        .map(|el| el.box_clone());

    let mut out = String::new();
    match command {
        Some(command) => {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                run_command(
                    &mut out, 
                    action.tool.clone(), 
                    command.box_clone(), 
                    context, 
                    action.args
                ).await
            })?;

        },
        None => {
            let error_str = format!("Error: No such tool named '{}'.", action.tool.clone());
            out.push_str(&error_str)
        }
    }

    Ok(out)
}