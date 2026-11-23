use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use eframe::egui::Context;

pub type Outcome<T> = Result<T, String>;

pub struct Task<T>(Receiver<Outcome<T>>);

impl<T: Send + 'static> Task<T> {
    pub fn spawn<E: ToString>(
        ctx: &Context,
        work: impl FnOnce() -> Result<T, E> + Send + 'static,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let _ = sender.send(work().map_err(|failure| failure.to_string()));
            ctx.request_repaint();
        });
        Self(receiver)
    }

    pub fn poll(slot: &mut Option<Self>) -> Option<Outcome<T>> {
        let outcome = match slot.as_ref()?.0.try_recv() {
            Ok(outcome) => outcome,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err("La operacion se interrumpio inesperadamente.".to_owned())
            }
        };
        *slot = None;
        Some(outcome)
    }
}
