use std::process::Command;
use std::sync::Arc;

use failure::ResultExt;
use slog::error;
use slog::Logger;

use crate::config::ServiceConfig;
use crate::ErrorKind;
use crate::Result;

/// Instantiate a service supervisor based on the provided configuration.
pub fn factory(logger: &Logger, service: ServiceConfig) -> Arc<dyn Supervisor> {
    let logger = logger.clone();
    match &service {
        ServiceConfig::Commands(options) => Arc::new(CommandSupervisor::commands(
            options.pid.clone(),
            options.start.clone(),
            options.stop.clone(),
            logger,
        )),
        ServiceConfig::Systemd(options) => {
            let service_name = options.service_name.clone();
            Arc::new(CommandSupervisor::systemd(service_name, logger))
        }
    }
}

/// Interface to the service supervisor.
pub trait Supervisor: Send + Sync {
    /// Returns the current service PID, if the service is running.
    fn pid(&self) -> Result<Option<String>>;

    /// Attempt to start the service.
    ///
    /// This method should return successfully if the service is already running.
    ///
    /// This method MAY block waiting for the process to start.
    fn start(&self) -> Result<()>;

    /// Attempt to stop the service.
    ///
    /// This method should return successfully if the service is already stopped.
    ///
    /// This method MAY block waiting for the process to stop.
    fn stop(&self) -> Result<()>;
}

/// Type alias to command functions for brevity.
type CmdFn<T> = Box<dyn Fn(&Logger) -> Result<T> + Send + Sync>;

/// Generic supervisor interface that executes commands to operate.
struct CommandSupervisor {
    cmd_pid: CmdFn<Option<String>>,
    cmd_start: CmdFn<()>,
    cmd_stop: CmdFn<()>,
    logger: Logger,
}

impl CommandSupervisor {
    fn commands(
        pid: Vec<String>,
        start: Vec<String>,
        stop: Vec<String>,
        logger: Logger,
    ) -> CommandSupervisor {
        CommandSupervisor {
            cmd_pid: commands_pid(pid),
            cmd_start: commands_act("start", start),
            cmd_stop: commands_act("stop", stop),
            logger,
        }
    }

    fn systemd(service_name: String, logger: Logger) -> CommandSupervisor {
        CommandSupervisor {
            cmd_pid: systemd_pid(service_name.clone()),
            cmd_start: systemd_start(service_name.clone()),
            cmd_stop: systemd_stop(service_name),
            logger,
        }
    }
}

impl Supervisor for CommandSupervisor {
    fn pid(&self) -> Result<Option<String>> {
        (self.cmd_pid)(&self.logger)
    }

    fn start(&self) -> Result<()> {
        (self.cmd_start)(&self.logger)
    }

    fn stop(&self) -> Result<()> {
        (self.cmd_stop)(&self.logger)
    }
}

/// Run a configured command.
fn commands_act(op: &'static str, cmd: Vec<String>) -> CmdFn<()> {
    Box::new(move |logger| {
        let action = Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .with_context(|_| ErrorKind::ServiceOpFailed(op))?;
        if !action.status.success() {
            let stderr = String::from_utf8(action.stderr)
                .with_context(|_| ErrorKind::ServiceOpFailed(op))?;
            error!(logger, "Failed to {} service", op; "stderr" => stderr);
            return Err(ErrorKind::ServiceOpFailed(op).into());
        }
        Ok(())
    })
}

/// Run a command and return the pid (stdout).
fn commands_pid(cmd: Vec<String>) -> CmdFn<Option<String>> {
    Box::new(move |logger| {
        let show = Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .with_context(|_| ErrorKind::ServiceOpFailed("pid"))?;
        if !show.status.success() {
            let stderr = String::from_utf8(show.stderr)
                .with_context(|_| ErrorKind::ServiceOpFailed("pid"))?;
            error!(logger, "Failed to check service pid"; "stderr" => stderr);
            return Err(ErrorKind::ServiceOpFailed("pid").into());
        }
        let stdout =
            String::from_utf8(show.stdout).with_context(|_| ErrorKind::ServiceOpFailed("pid"))?;
        if stdout == "" {
            return Ok(None);
        }
        Ok(Some(stdout))
    })
}

/// Fetch a systemd service PID, if the service is running.
fn systemd_pid(service_name: String) -> CmdFn<Option<String>> {
    Box::new(move |logger| {
        let show = Command::new("systemctl")
            .arg("show")
            .arg("--no-page")
            .arg("--property=MainPID")
            .arg("--property=SubState")
            .arg(&service_name)
            .output()
            .with_context(|_| ErrorKind::ServiceOpFailed("pid"))?;
        if !show.status.success() {
            let stderr = String::from_utf8(show.stderr)
                .with_context(|_| ErrorKind::ServiceOpFailed("pid"))?;
            error!(logger, "Failed to check service pid"; "stderr" => stderr);
            return Err(ErrorKind::ServiceOpFailed("pid").into());
        }
        let stdout =
            String::from_utf8(show.stdout).with_context(|_| ErrorKind::ServiceOpFailed("pid"))?;
        let mut pid = None;
        let mut running = false;
        for line in stdout.split('\n') {
            if line.starts_with("MainPID=") {
                pid = line.get(8..).map(ToString::to_string);
            }
            if line.starts_with("SubState=") {
                running = line == "SubState=running";
            }
        }
        if !running {
            return Ok(None);
        }
        Ok(pid)
    })
}

/// Request startup of a systemd service, if the service is running.
fn systemd_start(service_name: String) -> CmdFn<()> {
    Box::new(move |logger| {
        let start = Command::new("systemctl")
            .arg("start")
            .arg("--no-block")
            .arg(&service_name)
            .output()
            .with_context(|_| ErrorKind::ServiceOpFailed("start"))?;
        if !start.status.success() {
            let stderr = String::from_utf8(start.stderr)
                .with_context(|_| ErrorKind::ServiceOpFailed("start"))?;
            error!(logger, "Failed to start service"; "stderr" => stderr);
            return Err(ErrorKind::ServiceOpFailed("start").into());
        }
        Ok(())
    })
}

/// Request termination of a systemd service, if the service is running.
fn systemd_stop(service_name: String) -> CmdFn<()> {
    Box::new(move |logger| {
        let stop = Command::new("systemctl")
            .arg("stop")
            .arg("--no-block")
            .arg(&service_name)
            .output()
            .with_context(|_| ErrorKind::ServiceOpFailed("stop"))?;
        if !stop.status.success() {
            let stderr = String::from_utf8(stop.stderr)
                .with_context(|_| ErrorKind::ServiceOpFailed("stop"))?;
            error!(logger, "Failed to stop service"; "stderr" => stderr);
            return Err(ErrorKind::ServiceOpFailed("stop").into());
        }
        Ok(())
    })
}
