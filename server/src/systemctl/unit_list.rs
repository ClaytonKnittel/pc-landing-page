use super::commands::systemctl_capture;

/// Returns `true` if given `unit` exists,
/// ie., service could be or is actively deployed
/// and manageable by systemd
pub async fn exists(unit: &str) -> std::io::Result<bool> {
  let unit_list = list_units(None, None, Some(unit)).await?;
  Ok(!unit_list.is_empty())
}

/// Returns a `Vector` of `UnitList` structs extracted from systemctl listing.   
///  + type filter: optional `--type` filter
///  + state filter: optional `--state` filter
///  + glob filter: optional unit name filter
pub async fn list_units_full(
  type_filter: Option<&str>,
  state_filter: Option<&str>,
  glob: Option<&str>,
) -> std::io::Result<Vec<UnitList>> {
  let mut args = vec!["list-unit-files"];
  if let Some(filter) = type_filter {
    args.push("--type");
    args.push(filter)
  }
  if let Some(filter) = state_filter {
    args.push("--state");
    args.push(filter)
  }
  if let Some(glob) = glob {
    args.push(glob)
  }
  let mut result: Vec<UnitList> = Vec::new();
  let content = systemctl_capture(args).await?;
  let lines = content
    .lines()
    .filter(|line| line.contains('.') && !line.ends_with('.'));

  for l in lines {
    let parsed: Vec<&str> = l.split_ascii_whitespace().collect();
    let vendor_preset = match parsed[2] {
      "-" => None,
      "enabled" => Some(true),
      "disabled" => Some(false),
      _ => None,
    };
    result.push(UnitList {
      unit_file: parsed[0].to_string(),
      state: parsed[1].to_string(),
      vendor_preset,
    })
  }
  Ok(result)
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Implementation of list generated with
/// `systemctl list-unit-files`
pub struct UnitList {
  /// Unit name: `name.type`
  pub unit_file: String,
  /// Unit state
  pub state: String,
  /// Unit vendor preset
  pub vendor_preset: Option<bool>,
}

/// Returns a `Vector` of unit names extracted from systemctl listing.   
///  + type filter: optional `--type` filter
///  + state filter: optional `--state` filter
///  + glob filter: optional unit name filter
pub async fn list_units(
  type_filter: Option<&str>,
  state_filter: Option<&str>,
  glob: Option<&str>,
) -> std::io::Result<Vec<String>> {
  let list = list_units_full(type_filter, state_filter, glob).await;
  Ok(list?.iter().map(|n| n.unit_file.clone()).collect())
}

/// Returns list of services that are currently declared as disabled
pub async fn list_disabled_services() -> std::io::Result<Vec<String>> {
  list_units(Some("service"), Some("disabled"), None).await
}

/// Returns list of services that are currently declared as enabled
pub async fn list_enabled_services() -> std::io::Result<Vec<String>> {
  list_units(Some("service"), Some("enabled"), None).await
}
