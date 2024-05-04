use std::{
  io::{Error, ErrorKind},
  process::ExitStatus,
  str::FromStr,
};

use crate::error::ThreadSafeError;

use super::{
  commands::*,
  unit::{AsyncResult, AutoStartStatus, Doc, State, Type, Unit},
  unit_list::exists,
};
use async_trait::async_trait;
use futures_util::TryFutureExt;

/// Structure to describe a systemd `unit`
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SysUnit {
  /// Unit full name
  pub full_name: String,
  /// Unit name
  pub name: String,
  /// Unit type
  pub utype: Type,
  /// Optional unit description
  pub description: Option<String>,
  /// Current state
  pub state: State,
  /// Auto start feature
  pub auto_start: AutoStartStatus,
  /// `true` if Self is actively running
  pub active: bool,
  /// `true` if this unit is auto started by default,
  /// meaning, it should be manually disabled
  /// not to automatically start
  pub preset: bool,
  /// Configuration script loaded when starting this unit
  pub script: String,
  /// restart policy
  pub restart_policy: Option<String>,
  /// optionnal killmode info
  pub kill_mode: Option<String>,
  /// Optionnal process description (main tasklet "name")
  pub process: Option<String>,
  /// Optionnal process ID number (main tasklet pid)
  pub pid: Option<u64>,
  /// Running task(s) infos
  pub tasks: Option<u64>,
  /// Optionnal CPU load consumption infos
  pub cpu: Option<String>,
  /// Optionnal Memory consumption infos
  pub memory: Option<String>,
  /// mounted partition (`What`), if this is a `mount`/`automount` unit
  pub mounted: Option<String>,
  /// Mount point (`Where`), if this is a `mount`/`automount` unit
  pub mountpoint: Option<String>,
  /// Docs / `man` page(s) available for this unit
  pub docs: Option<Vec<Doc>>,
  /// wants attributes: list of other service / unit names
  pub wants: Option<Vec<String>>,
  /// wanted_by attributes: list of other service / unit names
  pub wanted_by: Option<Vec<String>>,
  /// also attributes
  pub also: Option<Vec<String>>,
  /// `before` attributes
  pub before: Option<Vec<String>>,
  /// `after` attributes
  pub after: Option<Vec<String>>,
  /// exec_start attribute: actual command line
  /// to be exected on `start` requests
  pub exec_start: Option<String>,
  /// exec_reload attribute, actual command line
  /// to be exected on `reload` requests
  pub exec_reload: Option<String>,
  /// If a command is run as transient service unit, it will be started and managed
  /// by the service manager like any other service, and thus shows up in the output
  /// of systemctl list-units like any other unit.
  pub transient: bool,
}

// TODO: Remove this lint fix
#[allow(clippy::if_same_then_else)]
impl SysUnit {
  /// Builds a new `Unit` structure by retrieving
  /// structure attributes with a `systemctl status $unit` call
  pub async fn from_systemctl(full_name: &str) -> std::io::Result<SysUnit> {
    if let Ok(false) = exists(full_name.to_string()).await {
      return Err(Error::new(
        ErrorKind::NotFound,
        format!("Unit or service \"{}\" does not exist", full_name),
      ));
    }
    let mut u = SysUnit::default();
    let status = status(full_name.to_string()).await?;
    let mut lines = status.lines();
    let next = lines.next().unwrap();
    let (_, rem) = next.split_at(3);
    let mut items = rem.split_ascii_whitespace();
    let name_raw = items.next().unwrap().trim();
    if let Some(delim) = items.next() {
      if delim.trim().eq("-") {
        // --> description string is provided
        let items: Vec<_> = items.collect();
        u.description = Some(itertools::join(&items, " "));
      }
    }
    let (name, utype_raw) = name_raw
      .rsplit_once('.')
      .expect("Unit is missing a Type, this should not happen!");
    // `type` is deduced from .extension
    u.utype = match Type::from_str(utype_raw) {
      Ok(t) => t,
      Err(e) => panic!("For {:?} -> {e}", name_raw),
    };
    let mut is_doc = false;
    for line in lines {
      let line = line.trim_start();
      if let Some(line) = line.strip_prefix("Loaded: ") {
        // Match and get rid of "Loaded: "
        if let Some(line) = line.strip_prefix("loaded ") {
          u.state = State::Loaded;
          let line = line.strip_prefix('(').unwrap();
          let line = line.strip_suffix(')').unwrap();
          let items: Vec<&str> = line.split(';').collect();
          u.script = items[0].trim().to_string();
          u.auto_start = match AutoStartStatus::from_str(items[1].trim()) {
            Ok(x) => x,
            Err(_) => AutoStartStatus::Disabled,
          };
          if items.len() > 2 {
            // preset is optionnal ?
            u.preset = items[2].trim().ends_with("enabled");
          }
        } else if line.starts_with("masked") {
          u.state = State::Masked;
        }
      } else if let Some(line) = line.strip_prefix("Transient: ") {
        if line == "yes" {
          u.transient = true
        }
      } else if line.starts_with("Active: ") {
        // skip that one
        // we already have .active() .inative() methods
        // to access this information
      } else if let Some(line) = line.strip_prefix("Docs: ") {
        is_doc = true;
        if let Ok(doc) = Doc::from_str(line) {
          u.docs.get_or_insert_with(Vec::new).push(doc);
        }
      } else if let Some(line) = line.strip_prefix("What: ") {
        // mountpoint infos
        u.mounted = Some(line.to_string())
      } else if let Some(line) = line.strip_prefix("Where: ") {
        // mountpoint infos
        u.mountpoint = Some(line.to_string());
      } else if let Some(line) = line.strip_prefix("Main PID: ") {
        // example -> Main PID: 787 (gpm)
        if let Some((pid, proc)) = line.split_once(' ') {
          u.pid = Some(pid.parse::<u64>().unwrap_or(0));
          u.process = Some(proc.replace(&['(', ')'][..], ""));
        };
      } else if let Some(line) = line.strip_prefix("Cntrl PID: ") {
        // example -> Main PID: 787 (gpm)
        if let Some((pid, proc)) = line.split_once(' ') {
          u.pid = Some(pid.parse::<u64>().unwrap_or(0));
          u.process = Some(proc.replace(&['(', ')'][..], ""));
        };
      } else if line.starts_with("Process: ") {
        //TODO: implement
        //TODO: parse as a Process item
        //let items : Vec<_> = line.split_ascii_whitespace().collect();
        //let proc_pid = u64::from_str_radix(items[1].trim(), 10).unwrap();
        //let cli;
        //Process: 640 ExecStartPre=/usr/sbin/sshd -t (code=exited, status=0/SUCCESS)
      } else if line.starts_with("CGroup: ") {
        //TODO: implement
        //LINE: "CGroup: /system.slice/sshd.service"
        //LINE: "└─1050 /usr/sbin/sshd -D"
      } else if line.starts_with("Tasks: ") {
        //TODO: implement
      } else if let Some(line) = line.strip_prefix("Memory: ") {
        u.memory = Some(line.trim().to_string());
      } else if let Some(line) = line.strip_prefix("CPU: ") {
        u.cpu = Some(line.trim().to_string())
      } else {
        // handling multi line cases
        if is_doc {
          let line = line.trim_start();
          if let Ok(doc) = Doc::from_str(line) {
            u.docs.get_or_insert_with(Vec::new).push(doc);
          }
        }
      }
    }

    if let Ok(content) = cat(name.to_string()).await {
      let line_tuple = content
        .lines()
        .filter_map(|line| line.split_once('=').to_owned());
      for (k, v) in line_tuple {
        let val = v.to_string();
        match k {
          "Wants" => u.wants.get_or_insert_with(Vec::new).push(val),
          "WantedBy" => u.wanted_by.get_or_insert_with(Vec::new).push(val),
          "Also" => u.also.get_or_insert_with(Vec::new).push(val),
          "Before" => u.before.get_or_insert_with(Vec::new).push(val),
          "After" => u.after.get_or_insert_with(Vec::new).push(val),
          "ExecStart" => u.exec_start = Some(val),
          "ExecReload" => u.exec_reload = Some(val),
          "Restart" => u.restart_policy = Some(val),
          "KillMode" => u.kill_mode = Some(val),
          _ => {}
        }
      }
    }

    u.active = is_active(name.to_string()).await?;
    u.full_name = full_name.to_string();
    u.name = name.to_string();
    Ok(u)
  }
}

#[async_trait]
impl Unit for SysUnit {
  fn name(&self) -> &str {
    &self.name
  }

  async fn refresh(&mut self) -> Result<(), Box<dyn ThreadSafeError>> {
    *self = Self::from_systemctl(&self.full_name).await?;
    Ok(())
  }

  /// Restarts Self by invoking `systemctl`
  fn restart(&mut self) -> AsyncResult<ExitStatus> {
    Box::pin(restart(self.name.clone()).map_err(|e| e.into()))
  }

  /// Starts Self by invoking `systemctl`
  fn start(&mut self) -> AsyncResult<ExitStatus> {
    Box::pin(start(self.name.clone()).map_err(|e| e.into()))
  }

  /// Stops Self by invoking `systemctl`
  fn stop(&mut self) -> AsyncResult<ExitStatus> {
    Box::pin(stop(self.name.clone()).map_err(|e| e.into()))
  }

  /// Reloads Self by invoking systemctl
  fn reload(&mut self) -> AsyncResult<ExitStatus> {
    Box::pin(reload(self.name.clone()).map_err(|e| e.into()))
  }

  /// Reloads or restarts Self by invoking systemctl
  fn reload_or_restart(&mut self) -> AsyncResult<ExitStatus> {
    Box::pin(reload_or_restart(self.name.clone()).map_err(|e| e.into()))
  }

  /// Enable Self to start at boot
  fn enable(&mut self) -> AsyncResult<ExitStatus> {
    Box::pin(enable(self.name.clone()).map_err(|e| e.into()))
  }

  /// Disable Self to start at boot
  fn disable(&mut self) -> AsyncResult<ExitStatus> {
    Box::pin(disable(self.name.clone()).map_err(|e| e.into()))
  }

  /// Returns verbose status for Self
  fn status(&self) -> AsyncResult<String> {
    Box::pin(status(self.name.clone()).map_err(|e| e.into()))
  }

  /// Returns `true` if Self is actively running
  fn is_active(&self) -> bool {
    self.active
  }

  /// `Isolate` Self, meaning stops all other units but
  /// self and its dependencies
  fn isolate(&mut self) -> AsyncResult<ExitStatus> {
    Box::pin(isolate(self.name.clone()).map_err(|e| e.into()))
  }

  /// `Freezes` Self, halts self and CPU load will
  /// no longer be dedicated to its execution.
  /// This operation might not be feasible.
  /// `unfreeze()` is the mirror operation
  fn freeze(&mut self) -> AsyncResult<ExitStatus> {
    Box::pin(freeze(self.name.clone()).map_err(|e| e.into()))
  }

  /// `Unfreezes` Self, exists halted state.
  /// This operation might not be feasible.
  fn unfreeze(&mut self) -> AsyncResult<ExitStatus> {
    Box::pin(unfreeze(self.name.clone()).map_err(|e| e.into()))
  }

  /// Returns `true` if given `unit` exists,
  /// ie., service could be or is actively deployed
  /// and manageable by systemd
  fn exists(&self) -> AsyncResult<bool> {
    Box::pin(exists(self.name.clone()).map_err(|e| e.into()))
  }
}
