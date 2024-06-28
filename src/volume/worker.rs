use crate::log::*;

use anyhow::Result;

pub enum WorkerMsg {}
pub enum ManagerMsg {
    Close,
}

use std::sync::mpsc::{Receiver, Sender};

pub fn work(lc: LC, recv: Receiver<ManagerMsg>, send: Sender<WorkerMsg>) -> Result<()> {
    info!(lc, "| work :: starting");

    for card in alsa::card::Iter::new() {
        match card {
            Ok(c) => {
                info!(lc, "| work :: card: {}", c.get_name()?);
                let ctl = alsa::hctl::HCtl::from_card(&c, false)?;
                ctl.load()?;

                //ctl.handle_events()?;
            }
            Err(err) => warn!(lc, "| work :: failed to enumerate card. error={err}"),
        }
    }

    info!(lc, "| work :: ending");
    Ok(())
}
