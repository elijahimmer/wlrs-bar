use super::utils::*;
use anyhow::{bail, Result};
use std::io::Read;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

#[derive(Debug)]
pub enum WorkerMsg {
    WorkspaceSetActive(WorkspaceID),
    WorkspaceCreate(WorkspaceID),
    WorkspaceDestroy(WorkspaceID),
    WorkspaceReset,
}

impl WorkerMsg {
    pub fn parse(cmd: &str, msg: &str) -> Result<Option<WorkerMsg>> {
        Ok(match cmd {
            "workspace" => Some(Self::WorkspaceSetActive(msg.parse()?)),
            "createworkspace" => Some(Self::WorkspaceCreate(msg.parse()?)),
            "destroyworkspace" => Some(Self::WorkspaceDestroy(msg.parse()?)),
            _ => {
                log::trace!("work :: cmd: '{cmd}' msg: '{msg}'");
                None
            }
        })
    }
}

#[derive(Debug)]
pub enum ManagerMsg {
    Close,
}

pub fn work(name: &str, recv: Receiver<ManagerMsg>, send: Sender<WorkerMsg>) -> Result<()> {
    let mut socket = open_hypr_socket(HyprSocket::Event)?;
    if let Err(err) = socket.set_nonblocking(true) {
        log::warn!("'{name}' | work :: couldn't set socket to non-blocking. error={err}");
    }

    send.send(WorkerMsg::WorkspaceReset)?;
    get_workspaces()?
        .into_iter()
        .try_for_each(|w| send.send(WorkerMsg::WorkspaceCreate(w)))?;

    send.send(WorkerMsg::WorkspaceSetActive(get_active_workspace()?))?;

    let mut buf = [0u8; 4096];

    loop {
        match recv.try_recv() {
            Ok(msg) => match msg {
                ManagerMsg::Close => {
                    log::debug!("'{name}' work :: told to close");
                    break;
                }
            },
            Err(TryRecvError::Disconnected) => {
                log::warn!("'{name}' work :: manager's send channel disconnected");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }

        let bytes_read = match socket.read(&mut buf) {
            Ok(b) => b,
            Err(err) => match err.kind() {
                std::io::ErrorKind::WouldBlock => continue,
                _ => bail!("'{name}' | work :: failed to read from socket: {err}"),
            },
        };

        String::from_utf8_lossy(&buf[..bytes_read])
            .lines()
            .filter_map(|line| line.find(">>").map(|idx| (&line[..idx], &line[idx + 2..])))
            .filter_map(|(cmd, msg)| {
                WorkerMsg::parse(cmd, msg)
                    .map_err(|err| log::warn!("Failed to parse WorkerMsg. err='{err}'"))
                    .ok()?
            })
            .try_for_each(|msg| send.send(msg))?;
    }

    Ok(())
}
