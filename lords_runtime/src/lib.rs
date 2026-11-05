#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::Thread;
use std::time::Instant;
use crossbeam_channel::{bounded, unbounded};
use log::warn;
use lords_sim::Simulation;

/// Number of real-time seconds per tick.
pub const TICK_INTERVAL: u64 = 1;

pub struct Runtime {
    state: Arc<RwLock<Arc<lords_sim::State>>>,
    cmd_tx: crossbeam_channel::Sender<Command>,
    paused: Arc<AtomicBool>,
    tick_thread: Thread,
}

impl Runtime {
    pub fn new(mut sim: Simulation) -> (Self, TickReceiver) {
        let state = Arc::new(RwLock::new(Arc::new(sim.state.clone())));
        let state_clone = Arc::clone(&state);
        let (cmd_tx, cmd_rx) = unbounded();
        let (tick_tx, tick_rx) = bounded(1);
        let tick_interval = std::time::Duration::from_secs(TICK_INTERVAL);
        let mut next_tick = Instant::now() + tick_interval;
        let paused = Arc::new(AtomicBool::new(false));
        let paused_clone = Arc::clone(&paused);

        let tick_thread = std::thread::spawn(move || {
            loop {
                while paused_clone.load(Ordering::Acquire) {
                    std::thread::park();
                    next_tick = Instant::now() + tick_interval;
                }

                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        Command::Reset => sim.state.time.zero(),
                    }
                }

                sim.tick();
                *state_clone.write().expect("Runtime state RwLock poisoned") = Arc::new(sim.state.clone());
                let _ = tick_tx.try_send(());

                let now = Instant::now();
                if next_tick > now {
                    std::thread::sleep(next_tick - now);
                } else if next_tick < now {
                    let tick_overtime_ms = (now - next_tick).as_millis();
                    warn!("Tick was {tick_overtime_ms} ms late")
                }

                next_tick = Instant::now() + tick_interval;
            }
        }).thread().clone();

        (Self { state, cmd_tx, paused, tick_thread }, TickReceiver { 0: tick_rx })
    }

    /// Returns a snapshot of the state as of the latest tick.
    pub fn state(&self) -> Arc<lords_sim::State> {
        Arc::clone(&self.state.read().expect("Runtime state RwLock poisoned"))
    }

    pub fn pause(&self)  {
        self.paused.store(true, Ordering::Release);
    }
    pub fn resume(&self) {
        self.paused.store(false, Ordering::Release);
        self.tick_thread.unpark();
    }
    pub fn reset(&self)  {
        let _ = self.cmd_tx.send(Command::Reset);
        self.resume();
    }
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Acquire)
    }
}

pub struct TickReceiver(crossbeam_channel::Receiver<()>);

impl TickReceiver {
    /// Blocks the current thread until a tick is received.
    pub fn recv_is_ok(&self) -> bool {
        self.0.recv().is_ok()
    }
}

pub enum Command {
    Reset,
}
