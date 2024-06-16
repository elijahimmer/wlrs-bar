use anyhow::{anyhow, Result};
use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

pub const COMMAND_SOCKET: &str = ".socket.sock";
pub const EVENT_SOCKET: &str = ".socket2.sock";

pub type WorkspaceID = i32;

pub enum HyprSocket {
    Command,
    Event,
}

pub enum Command {
    MoveToWorkspace(WorkspaceID),
    ActiveWorkspace,
    Workspaces,
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

pub fn send_hypr_command(command: Command) -> Result<Box<str>> {
    let mut socket = open_hypr_socket(HyprSocket::Command)?;
    write!(socket, "{command}")?;
    socket.flush()?;

    let mut res = String::new();

    socket.read_to_string(&mut res)?;
    let res = res.trim();

    if res == "unknown request" {
        Err(anyhow!("Invaid Hyprland Command! '{command}'"))
    } else {
        Ok(res.into())
    }
}

const WKSP_CMD_START: &str = "workspace ID ";
const WKSP_CMD_LEN: usize = WKSP_CMD_START.len();
pub fn get_active_workspace() -> Result<WorkspaceID> {
    let res = &send_hypr_command(Command::ActiveWorkspace)?[WKSP_CMD_LEN..];

    match res.find(' ') {
        Some(pos) => Ok(res[..pos].parse()?),
        None => Err(anyhow!("Failed to parse Hyprctl Response")),
    }
}

pub fn get_workspaces() -> Result<Vec<WorkspaceID>> {
    let res = send_hypr_command(Command::Workspaces)?;
    assert!(res.len() > WKSP_CMD_LEN);

    res
        .lines()
        .filter(|l| l.starts_with(WKSP_CMD_START))
        .
}

//pub fn jumpstart_workspaces() -> Result<Vec<Workspace>> {
//    let res = send_hypr_command_read("workspaces")?;
//
//    let mut vec = vec![];
//    for line in res.lines() {
//        if line.starts_with(CMD_LINE_START) {
//            let pos = line[CMD_LINE_LEN..]
//                .find(' ')
//                .ok_or(anyhow!("Failed to parse Hyprland response."))?;
//
//            vec.push(create_workspace(
//                line[CMD_LINE_LEN..(CMD_LINE_LEN + pos)].parse()?,
//            ));
//        }
//    }
//
//    vec.sort_unstable_by_key(|e| e.0);
//
//    Ok(vec)
//}
//
//pub fn jumpstart_active_workspace() -> Result<i32> {
//    let res = send_hypr_command_read("activeworkspace")?;
//
//    let pos = match res[CMD_LINE_LEN..].find(' ') {
//        Some(p) => p,
//        None => {
//            return Err(anyhow!("Failed to parse Hyprctl Response"));
//        }
//    };
//    Ok(res[CMD_LINE_LEN..(CMD_LINE_LEN + pos)].parse()?)
//}

//pub fn send_hypr_command(command: HyprCommand) -> Result<()> {
//    let mut socket = open_hypr_socket(HyprSocket::Command)?;
//    command.write_to(&mut socket)?;
//    socket.flush()?;
//
//    let mut buf = [0; 16];
//
//    let size = socket.read(&mut buf)?;
//
//    if buf[..size] == *b"unknown request" {
//        Err(anyhow!("Invaid Hyprland Command!"))
//    } else {
//        Ok(())
//    }
//}
