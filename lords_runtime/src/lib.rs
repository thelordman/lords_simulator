#![warn(clippy::all, rust_2018_idioms)]

use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;
use lords_sim::Simulation;

/// Number of real-time seconds per tick.
pub const TICK_INTERVAL: u64 = 1;

pub struct Runtime {
    pub state: Arc<RwLock<lords_sim::State>>,
    cmd_tx: Sender<Command>,
}

impl Runtime {
    pub fn new(mut sim: Simulation) -> (Self, Receiver<()>) {
        let state = Arc::new(RwLock::new(sim.state.clone()));
        let state_clone = Arc::clone(&state);
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let (tick_tx, tick_rx) = mpsc::channel::<()>();
        let tick_interval = std::time::Duration::from_secs(TICK_INTERVAL);
        let mut next_tick = Instant::now() + tick_interval;

        std::thread::spawn(move || {
            let mut paused = false;
            loop {
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        Command::Pause  => paused = true,
                        Command::Resume => paused = false,
                        Command::Reset  => {
                            sim.state.time.zero(); paused = false;
                        }
                    }
                }

                if !paused {
                    sim.tick();
                    *state_clone.write().expect("Runtime state RwLock poisoned") = sim.state.clone();
                    let _ = tick_tx.send(());
                }

                let now = Instant::now();
                if next_tick > now {
                    std::thread::sleep(next_tick - now);
                }

                next_tick += tick_interval;
            }
        });

        (Self { state, cmd_tx }, tick_rx)
    }

    pub fn pause(&self)  {
        let _ = self.cmd_tx.send(Command::Pause);
    }
    pub fn resume(&self) {
        let _ = self.cmd_tx.send(Command::Resume);
    }
    pub fn reset(&self)  {
        let _ = self.cmd_tx.send(Command::Reset);
    }
}

pub enum Command {
    Pause,
    Resume,
    Reset,
}
