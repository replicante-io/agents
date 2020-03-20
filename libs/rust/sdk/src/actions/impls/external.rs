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
use crate::config::ExternalActionConfig;
use crate::store::Transaction;
use crate::AgentContext;
use crate::ErrorKind;
use crate::Result;

pub fn register(context: &AgentContext) -> Result<()> {
    debug!(context.logger, "Registering configured external actions");
    for (kind, config) in &context.config.external_actions {
        if config.action.is_empty() {
            return Err(ErrorKind::Initialisation(format!(
                "empty action command for external_actions.{}",
                kind,
            ))
            .into());
        }
        if config.check.is_empty() {
            return Err(ErrorKind::Initialisation(format!(
                "empty check command for external_actions.{}",
                kind
            ))
            .into());
        }
        let kind = format!("external.agent.replicante.io/{}", kind);
        let action = ExternalAction::new(kind, config.clone(), context.logger.clone());
        ACTIONS::register_reserved(action);
    }
    Ok(())
}

/// Execute user-defined actions by executing commands.
#[derive(Debug)]
pub struct ExternalAction {
    config: ExternalActionConfig,
    kind: String,
    logger: Logger,
}

impl ExternalAction {
    pub fn new(kind: String, config: ExternalActionConfig, logger: Logger) -> ExternalAction {
        ExternalAction {
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
        let output = self.exec(record, &self.config.check, ErrorKind::ExternalActionCheck)?;
        let action_id = ActionRecordView::id(record);
        let stdout =
            String::from_utf8(output.stdout).unwrap_or_else(|_| "{binary blob}".to_string());
        if !output.status.success() {
            let stderr =
                String::from_utf8(output.stderr).unwrap_or_else(|_| "{binary blob}".to_string());
            let error = ErrorKind::ExternalActionCheckResult(action_id, stdout, stderr);
            return Err(error.into());
        }
        let report: ExternalActionReport = serde_json::from_str(&stdout)
            .with_context(|_| ErrorKind::ExternalActionCheckDecode(action_id))?;
        match report {
            ExternalActionReport::Failed(report) => tx.action().transition(
                record,
                ActionState::Failed,
                serde_json::to_value(&report).expect("report serialisation must succeed"),
                span.map(|span| span.context().clone()),
            ),
            ExternalActionReport::Finished => tx.action().transition(
                record,
                ActionState::Done,
                None,
                span.map(|span| span.context().clone()),
            ),
            ExternalActionReport::Running => Ok(()),
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
        let info = ExternalActionInfo {
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
        let output = self.exec(record, &self.config.action, ErrorKind::ExternalActionStart)?;
        let stdout =
            String::from_utf8(output.stdout).unwrap_or_else(|_| "{binary blob}".to_string());
        let action_id = ActionRecordView::id(record);
        debug!(
            self.logger,
            "External action started";
            "action_id" => %action_id,
            "kind" => &self.kind,
            "stdout" => &stdout,
        );
        if !output.status.success() {
            let stderr =
                String::from_utf8(output.stderr).unwrap_or_else(|_| "{binary blob}".to_string());
            let error = ErrorKind::ExternalActionExec(action_id, stdout, stderr);
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

impl Action for ExternalAction {
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
struct ExternalActionInfo {
    args: Json,
    headers: HashMap<String, String>,
    id: Uuid,
    kind: String,
}

/// Expected outcomes from a check command.
#[derive(Serialize, Deserialize)]
#[serde(tag = "status")]
enum ExternalActionReport {
    #[serde(rename = "failed")]
    Failed(ExternalActionFailed),

    #[serde(rename = "finished")]
    Finished,

    #[serde(rename = "running")]
    Running,
}

/// If the action failed, information about the error is expected.
#[derive(Serialize, Deserialize)]
struct ExternalActionFailed {
    error: Option<String>,
}
