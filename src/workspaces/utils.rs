use anyhow::{anyhow, Result};
use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

pub const COMMAND_SOCKET: &str = ".socket.sock";
pub const EVENT_SOCKET: &str = ".socket2.sock";

pub type WorkspaceID = i32;

#[derive(Debug)]
pub enum HyprSocket {
    Command,
    Event,
}

#[derive(Debug)]
pub enum Command {
    MoveToWorkspace(WorkspaceID),
    ActiveWorkspace,
    Workspaces,
}

use std::fmt::{Display, Error as FmtError, Formatter};
impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            Command::MoveToWorkspace(wid) => write!(f, "dispatch workspace {wid}"),
            Command::ActiveWorkspace => write!(f, "activeworkspace"),
            Command::Workspaces => write!(f, "workspaces"),
        }
    }
}

pub fn open_hypr_socket(socket: HyprSocket) -> Result<UnixStream> {
    let xdg_dir = env::var("XDG_RUNTIME_DIR")?;
    let his = env::var("HYPRLAND_INSTANCE_SIGNATURE")?;

    let socket_file = match socket {
        HyprSocket::Command => COMMAND_SOCKET,
        HyprSocket::Event => EVENT_SOCKET,
    };

    Ok(UnixStream::connect(format!(
        "{xdg_dir}/hypr/{his}/{socket_file}"
    ))?)
}

pub fn send_hypr_command(command: Command) -> Result<Box<str>> {
    let mut socket = open_hypr_socket(HyprSocket::Command)?;
    write!(socket, "{command}")?;
    socket.flush()?;

    let mut res = String::new();

    socket.read_to_string(&mut res)?;
    let res = res.trim();

    if res == "unknown request" {
        Err(anyhow!("Invaid Hyprland command '{command}'"))
    } else {
        Ok(res.into())
    }
}

const WKSP_CMD_START: &str = "workspace ID ";
const WKSP_CMD_LEN: usize = WKSP_CMD_START.len();

pub fn get_active_workspace() -> Result<WorkspaceID> {
    send_hypr_command(Command::ActiveWorkspace).and_then(|l| get_workspace_id(&l))
}

pub fn get_workspaces() -> Result<Vec<WorkspaceID>> {
    send_hypr_command(Command::Workspaces)?
        .lines()
        .filter(|l| l.starts_with(WKSP_CMD_START))
        .map(get_workspace_id)
        .collect::<Result<Vec<_>>>()
        .map(|mut v| {
            v.sort();
            v
        })
}

fn get_workspace_id(line: &str) -> Result<WorkspaceID> {
    assert!(line.starts_with(WKSP_CMD_START));
    line[WKSP_CMD_LEN..]
        .find(' ')
        .ok_or(anyhow!("Invalid Workspace Response '{line}'"))
        .and_then(|idx| Ok(line[WKSP_CMD_LEN..][..idx].parse()?))
}

const ALPHA_CHAR: u32 = 'Α' as u32 - 1;

pub fn map_workspace_id(id: WorkspaceID) -> String {
    match id {
        i @ 1..=17 => match char::from_u32(ALPHA_CHAR + i as u32) {
            Some(ch) => ch.to_string(),
            None => {
                log::warn!("Failed to map workspace to symbol: i={i}");
                format!("{}", i)
            }
        },
        // I needed to split this because there is a reserved character between rho and sigma.
        i @ 18..=24 => match char::from_u32((ALPHA_CHAR + 1) + i as u32) {
            Some(ch) => ch.to_string(),
            None => {
                log::warn!("Failed to map workspace to symbol: i={i}");
                format!("{}", i)
            }
        },
        i => format!("{}", i),
    }
}
