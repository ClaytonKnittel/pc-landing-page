use std::error;

use systemctl::Unit;

const MC_SERVER_SERVICE: &str = "mc_server.service";

pub fn mc_server_status() -> Result<Unit, Box<dyn error::Error>> {
  Unit::from_systemctl(MC_SERVER_SERVICE).map_err(Box::from)
}
