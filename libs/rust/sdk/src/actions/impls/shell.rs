use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

use failure::ResultExt;
use opentracingrust::Span;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value as Json;
use slog::debug;
use slog::Logger;
use uuid::Uuid;

use crate::actions::Action;
use crate::actions::ActionDescriptor;
use crate::actions::ActionRecordView;
use crate::actions::ActionState;
use crate::actions::ActionValidity;
use crate::actions::ACTIONS;
use crate::config::ShellActionConfig;
use crate::store::Transaction;
use crate::AgentContext;
use crate::ErrorKind;
use crate::Result;

pub fn register(context: &AgentContext) -> Result<()> {
    debug!(context.logger, "Registering configured shell actions");
    for (kind, config) in &context.config.shell_actions {
        if config.action.is_empty() {
            return Err(ErrorKind::Initialisation(format!(
                "empty action command for shell_actions.{}",
                kind,
            ))
            .into());
        }
        if config.check.is_empty() {
            return Err(ErrorKind::Initialisation(format!(
                "empty check command for shell_actions.{}",
                kind
            ))
            .into());
        }
        let kind = format!("shell.replicante.io/{}", kind);
        let action = ShellAction::new(kind, config.clone(), context.logger.clone());
        ACTIONS::register_reserved(action);
    }
    Ok(())
}

/// Execute user-defined actions by executing commands.
#[derive(Debug)]
pub struct ShellAction {
    config: ShellActionConfig,
    kind: String,
    logger: Logger,
}

impl ShellAction {
    pub fn new(kind: String, config: ShellActionConfig, logger: Logger) -> ShellAction {
        ShellAction {
            config,
            kind,
            logger,
        }
    }

    fn check_action(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        let output = self.exec(record, &self.config.check, ErrorKind::ShellActionCheck)?;
        let action_id = ActionRecordView::id(record);
        let stdout =
            String::from_utf8(output.stdout).unwrap_or_else(|_| "{binary blob}".to_string());
        if !output.status.success() {
            let stderr =
                String::from_utf8(output.stderr).unwrap_or_else(|_| "{binary blob}".to_string());
            let error = ErrorKind::ShellActionCheckResult(action_id, stdout, stderr);
            return Err(error.into());
        }
        let report: ShellActionReport = serde_json::from_str(&stdout)
            .with_context(|_| ErrorKind::ShellActionCheckDecode(action_id))?;
        match report {
            ShellActionReport::Failed(report) => tx.action().transition(
                record,
                ActionState::Failed,
                serde_json::to_value(&report).expect("report serialisation must succeed"),
                span.map(|span| span.context().clone()),
            ),
            ShellActionReport::Finished => tx.action().transition(
                record,
                ActionState::Done,
                None,
                span.map(|span| span.context().clone()),
            ),
            ShellActionReport::Running => Ok(()),
        }
    }

    fn exec<F>(
        &self,
        record: &dyn ActionRecordView,
        command: &[String],
        error_kind: F,
    ) -> Result<Output>
    where
        F: Fn(String, Uuid) -> ErrorKind,
    {
        let action_id = ActionRecordView::id(record);
        let info = ShellActionInfo {
            args: record.args().clone(),
            headers: ActionRecordView::headers(record).clone(),
            id: action_id,
            kind: self.kind.clone(),
        };
        let info =
            serde_json::to_vec(&info).with_context(|_| error_kind(self.kind.clone(), action_id))?;
        let cmd = &command[0];
        let args = &command[1..];
        let mut child = Command::new(cmd)
            .args(args)
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .with_context(|_| error_kind(self.kind.clone(), action_id))?;
        {
            let stdin = child.stdin.as_mut().expect("failed to open stdin");
            stdin
                .write_all(&info)
                .with_context(|_| error_kind(self.kind.clone(), action_id))?;
        }
        let output = child
            .wait_with_output()
            .with_context(|_| error_kind(self.kind.clone(), action_id))?;
        Ok(output)
    }

    fn start_action(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        let output = self.exec(record, &self.config.action, ErrorKind::ShellActionStart)?;
        let stdout =
            String::from_utf8(output.stdout).unwrap_or_else(|_| "{binary blob}".to_string());
        let action_id = ActionRecordView::id(record);
        debug!(
            self.logger,
            "Shell action exec'ed";
            "action_id" => %action_id,
            "kind" => &self.kind,
            "stdout" => &stdout,
        );
        if !output.status.success() {
            let stderr =
                String::from_utf8(output.stderr).unwrap_or_else(|_| "{binary blob}".to_string());
            let error = ErrorKind::ShellActionExec(action_id, stdout, stderr);
            return Err(error.into());
        }
        tx.action().transition(
            record,
            ActionState::Running,
            None,
            span.map(|span| span.context().clone()),
        )
    }
}

impl Action for ShellAction {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            description: self.config.description.clone(),
            kind: self.kind.clone(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        match record.state() {
            ActionState::New => self.start_action(tx, record, span)?,
            ActionState::Running => self.check_action(tx, record, span)?,
            _ => (),
        }
        Ok(())
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct ShellActionInfo {
    args: Json,
    headers: HashMap<String, String>,
    id: Uuid,
    kind: String,
}

/// Expected outcomes from a check command.
#[derive(Serialize, Deserialize)]
#[serde(tag = "status")]
enum ShellActionReport {
    #[serde(rename = "failed")]
    Failed(ShellActionFailed),

    #[serde(rename = "finished")]
    Finished,

    #[serde(rename = "running")]
    Running,
}

/// If the action failed, information about the error is expected.
#[derive(Serialize, Deserialize)]
struct ShellActionFailed {
    error: Option<String>,
}
