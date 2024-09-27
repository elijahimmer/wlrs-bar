use super::utils::*;
use crate::log::*;

use std::io::Read;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use thiserror::Error;

#[derive(Debug)]
pub enum WorkerMsg {
    WorkspaceSetActive(WorkspaceID),
    WorkspaceCreate(WorkspaceID),
    WorkspaceDestroy(WorkspaceID),
    WorkspaceReset,
}

impl WorkerMsg {
    pub fn parse(cmd: &str, msg: &str) -> Result<Option<WorkerMsg>, std::num::ParseIntError> {
        Ok(match cmd {
            "workspace" => Some(Self::WorkspaceSetActive(msg.parse()?)),
            "createworkspace" => Some(Self::WorkspaceCreate(msg.parse()?)),
            "destroyworkspace" => Some(Self::WorkspaceDestroy(msg.parse()?)),
            _ => None,
        })
    }
}

#[derive(Debug)]
pub enum ManagerMsg {
    Close,
}

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error(transparent)]
    OpenHyprSocket(#[from] OpenHyprSocketError),
    #[error(transparent)]
    GetWorkspace(#[from] GetWorkspaceError),
    #[error("Failed to use Hyprland socket with `{0}`")]
    SocketError(#[from] std::io::Error),
    #[error("Failed to send message to Manager thread with `{0}`")]
    ManagerMsgError(#[from] std::sync::mpsc::SendError<WorkerMsg>),
}

pub fn work(
    lc: LC,
    recv: Receiver<ManagerMsg>,
    send: Sender<WorkerMsg>,
) -> Result<(), WorkerError> {
    let mut socket = open_hypr_socket(HyprSocket::Event)?;
    if let Err(err) = socket.set_nonblocking(true) {
        warn!(
            lc,
            "work :: couldn't set socket to non-blocking. error={err}"
        );
    }

    send.send(WorkerMsg::WorkspaceReset)?;

    let _ = get_workspaces()?
        .into_iter()
        .try_for_each(|w| send.send(WorkerMsg::WorkspaceCreate(w)))
        .inspect_err(|err| warn!(lc, "work :: failed to get initial workspaces with `{err}`"));

    send.send(WorkerMsg::WorkspaceSetActive(get_active_workspace()?))?;

    let mut buf = [0u8; 4096];

    loop {
        match recv.try_recv() {
            Ok(msg) => match msg {
                ManagerMsg::Close => {
                    info!(lc, "work :: told to close");
                    break;
                }
            },
            Err(TryRecvError::Disconnected) => {
                warn!(lc, "work :: manager's send channel disconnected");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }

        std::thread::sleep(std::time::Duration::from_millis(50));

        let bytes_read = match socket.read(&mut buf) {
            Ok(b) => b,
            Err(err) => match err.kind() {
                std::io::ErrorKind::WouldBlock => continue,
                _ => return Err(WorkerError::SocketError(err)),
            },
        };

        String::from_utf8_lossy(&buf[..bytes_read])
            .lines()
            .filter_map(|line| line.find(">>").map(|idx| (&line[..idx], &line[idx + 2..])))
            .filter_map(|(cmd, msg)| {
                println!("cmd: {cmd} - msg: {msg}");
                WorkerMsg::parse(cmd, msg)
                    .map_err(|err| warn!(lc, "| work :: Failed to parse WorkerMsg. error='{err}'"))
                    .ok()?
            })
            .try_for_each(|msg| send.send(msg))?;
    }

    Ok(())
}
