use super::utils::*;
use anyhow::Result;
use std::io::Read;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

#[derive(Debug)]
pub enum WorkerMsg {
    WorkspaceSetActive(WorkspaceID),
    WorkspaceCreate(WorkspaceID),
    WorkspaceDestroy(WorkspaceID),
    WorkspaceReset,
}

#[derive(Debug)]
pub enum ManagerMsg {
    Close,
}

pub fn work(name: &str, recv: Receiver<ManagerMsg>, send: Sender<WorkerMsg>) -> Result<()> {
    let mut socket = open_hypr_socket(HyprSocket::Event)?;
    //socket.set_nonblocking(true)?;

    std::thread::sleep(std::time::Duration::from_secs(1));

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
                    log::debug!("'name', worker told to close");
                    break;
                }
            },
            Err(TryRecvError::Disconnected) => {
                log::warn!("'name', manager's send channel disconnected");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }

        let bytes_read = socket.read(&mut buf)?;
        //let valid_bytes = bytes_read()

        log::trace!("'{name}', worker read {bytes_read:?} bytes");
    }

    Ok(())
}
