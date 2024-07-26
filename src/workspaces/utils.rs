use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use thiserror::Error;

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
    ActiveWorkspace,
    Workspaces,
    MoveToWorkspace(WorkspaceID),
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

#[derive(Error, Debug)]
pub enum OpenHyprSocketError {
    #[error("'XDG_RUNTIME_DIR' {0}")]
    XDGRuntimeDirNotSet(std::env::VarError),
    #[error("'HYPRLAND_INSTANCE_SIGNATURE' {0}")]
    HISNotSet(std::env::VarError),
    #[error("Failed to connect to Hyprland socket with `{0}`")]
    UnixStreamConnect(#[from] std::io::Error),
}

pub fn open_hypr_socket(socket: HyprSocket) -> Result<UnixStream, OpenHyprSocketError> {
    let xdg_dir = env::var("XDG_RUNTIME_DIR").map_err(OpenHyprSocketError::XDGRuntimeDirNotSet)?;
    let his = env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(OpenHyprSocketError::HISNotSet)?;

    let socket_file = match socket {
        HyprSocket::Command => COMMAND_SOCKET,
        HyprSocket::Event => EVENT_SOCKET,
    };

    UnixStream::connect(format!("{xdg_dir}/hypr/{his}/{socket_file}"))
        .map_err(OpenHyprSocketError::UnixStreamConnect)
}

#[derive(Error, Debug)]
pub enum SendHyprCommandError {
    #[error(transparent)]
    OpenHyprSocket(#[from] OpenHyprSocketError),
    #[error("Failed to use hyprland socket with `{0}`")]
    IOError(#[from] std::io::Error),
    #[error("Invalid Hyprland command (submit bug report). command=`{0}`")]
    InvalidCommand(Command),
}

pub fn send_hypr_command(command: Command) -> Result<Box<str>, SendHyprCommandError> {
    let mut socket = open_hypr_socket(HyprSocket::Command)?;
    write!(socket, "{command}")?;
    socket.flush()?;

    let mut res = String::new();

    socket.read_to_string(&mut res)?;
    let res = res.trim();

    if res == "unknown request" {
        Err(SendHyprCommandError::InvalidCommand(command))
    } else {
        Ok(res.into())
    }
}

const WKSP_CMD_START: &str = "workspace ID ";
const WKSP_CMD_LEN: usize = WKSP_CMD_START.len();

#[derive(Error, Debug)]
pub enum GetWorkspaceError {
    #[error(transparent)]
    SendHyprCommand(#[from] SendHyprCommandError),
    #[error(transparent)]
    GetWorkspaceId(#[from] GetWorkspaceIdError),
}

pub fn get_active_workspace() -> Result<WorkspaceID, GetWorkspaceError> {
    Ok(get_workspace_id(&send_hypr_command(
        Command::ActiveWorkspace,
    )?)?)
}

pub fn get_workspaces() -> Result<Vec<WorkspaceID>, GetWorkspaceError> {
    Ok(send_hypr_command(Command::Workspaces)?
        .lines()
        .filter(|l| l.starts_with(WKSP_CMD_START))
        .map(get_workspace_id)
        .collect::<Result<Vec<_>, GetWorkspaceIdError>>()
        .map(|mut v| {
            v.sort();
            v
        })?)
}

#[derive(Error, Debug)]
pub enum GetWorkspaceIdError {
    #[error("Hyprland's workspace response was invalid (submit bug report).")]
    InvalidWorkspaceResponse,
    #[error("Failed to parse the workspace id with `{0}`")]
    FailedToParseId(std::num::ParseIntError),
}

fn get_workspace_id(line: &str) -> Result<WorkspaceID, GetWorkspaceIdError> {
    assert!(line.starts_with(WKSP_CMD_START));
    line[WKSP_CMD_LEN..]
        .find(' ')
        .ok_or(GetWorkspaceIdError::InvalidWorkspaceResponse)
        .and_then(|idx| {
            line[WKSP_CMD_LEN..][..idx]
                .parse()
                .map_err(GetWorkspaceIdError::FailedToParseId)
        })
}

const ALPHA_CHAR: u32 = 'Α' as u32 - 1;

pub fn map_workspace_id(id: WorkspaceID) -> String {
    match id {
        i @ 1..=17 => char::from_u32(ALPHA_CHAR + i as u32).unwrap().to_string(),
        // I needed to split this because there is a reserved character between rho and sigma.
        i @ 18..=24 => char::from_u32((ALPHA_CHAR + 1) + i as u32)
            .unwrap()
            .to_string(),
        i => format!("{}", i),
    }
}
