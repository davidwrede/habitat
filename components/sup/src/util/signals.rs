// Copyright (c) 2016 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Traps and notifies UNIX signals.
//!
//! Start's another thread which you can subscribe to which traps UNIX signals
//! sent to the running process and notifies the receiver channel of a caught
//! `signals::Signal`.

use std::sync::{Once, ONCE_INIT};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering, ATOMIC_USIZE_INIT, ATOMIC_BOOL_INIT};

use wonder::actor;
use wonder::actor::{ActorSender, HandleResult, InitResult, StopReason};

use error::{Error, Result, SupError};

static LOGKEY: &'static str = "US";

const TIMEOUT_MS: u64 = 30;
static INIT: Once = ONCE_INIT;
static mut ALIVE: AtomicBool = ATOMIC_BOOL_INIT;
// True when we have caught a signal
static mut CAUGHT: AtomicBool = ATOMIC_BOOL_INIT;
// Stores the value of the signal we caught
static mut SIGNAL: AtomicUsize = ATOMIC_USIZE_INIT;

// Functions from POSIX libc.
extern "C" {
    fn signal(sig: u32, cb: unsafe extern "C" fn(u32)) -> unsafe extern "C" fn(u32);
    fn kill(pid: i32, sig: u32) -> u32;
}

unsafe extern "C" fn handle_signal(signal: u32) {
    CAUGHT.store(true, Ordering::SeqCst);
    SIGNAL.store(signal as usize, Ordering::SeqCst);
}

#[derive(Debug)]
pub enum Message {
    Signal(Signal),
    Stop,
    Ok,
}

/// `i32` representation of each Unix Signal of interest.
#[derive(Debug, Clone)]
pub enum Signal {
    /// terminate process - terminal line hangup
    SIGHUP = 1,
    /// terminate process - interrupt program
    SIGINT = 2,
    /// create core image - quit program
    SIGQUIT = 3,
    /// Kill a process
    SIGKILL = 9,
    /// terminate process - real-time timer expired
    SIGALRM = 14,
    /// terminate process - software termination signal
    SIGTERM = 15,
    /// terminate process - User defined signal 1
    SIGUSR1 = 30,
    /// terminate process - User defined signal 2
    SIGUSR2 = 31,
}

/// Thread worker that traps UNIX signals and sends a `Signal` down the receiver
/// channel representing the trapped UNIX signal.
pub struct SignalNotifier;

impl SignalNotifier {
    pub fn stop(actor: &actor::Actor<Message>) -> Result<()> {
        match actor.call(Message::Stop) {
            Ok(_) => Ok(()),
            Err(err) => Err(SupError::from(err)),
        }
    }
}

impl actor::GenServer for SignalNotifier {
    type T = Message;
    type S = ();
    type E = SupError;

    fn init(&self, _tx: &ActorSender<Self::T>, _: &mut Self::S) -> InitResult<Self::E> {
        unsafe {
            INIT.call_once(|| {
                self::set_signal_handlers();
                CAUGHT.store(false, Ordering::SeqCst);
                SIGNAL.store(0 as usize, Ordering::SeqCst);
            });
            if ALIVE.compare_and_swap(false, true, Ordering::Relaxed) {
                return Err(sup_error!(Error::SignalNotifierStarted));
            }
        }
        Ok(Some(TIMEOUT_MS))
    }

    fn handle_call(&self,
                   message: Self::T,
                   _: &ActorSender<Self::T>,
                   _: &ActorSender<Self::T>,
                   _: &mut Self::S)
                   -> HandleResult<Self::T> {
        match message {
            Message::Stop => HandleResult::Stop(StopReason::Normal, Some(Message::Ok)),
            msg => {
                HandleResult::Stop(StopReason::Fatal(format!("unexpected call message: {:?}", msg)),
                                   Some(Message::Ok))
            }
        }
    }

    fn handle_timeout(&self,
                      tx: &ActorSender<Self::T>,
                      _me: &ActorSender<Self::T>,
                      _: &mut Self::S)
                      -> HandleResult<Self::T> {
        unsafe {
            if CAUGHT.load(Ordering::SeqCst) {
                match SIGNAL.load(Ordering::SeqCst) {
                    signal if signal == Signal::SIGHUP as usize => {
                        self::send_signal(tx, Signal::SIGHUP)
                    }
                    signal if signal == Signal::SIGINT as usize => {
                        self::send_signal(tx, Signal::SIGINT)
                    }
                    signal if signal == Signal::SIGQUIT as usize => {
                        self::send_signal(tx, Signal::SIGQUIT)
                    }
                    signal if signal == Signal::SIGALRM as usize => {
                        self::send_signal(tx, Signal::SIGALRM)
                    }
                    signal if signal == Signal::SIGTERM as usize => {
                        self::send_signal(tx, Signal::SIGTERM)
                    }
                    signal if signal == Signal::SIGUSR1 as usize => {
                        self::send_signal(tx, Signal::SIGUSR1)
                    }
                    signal if signal == Signal::SIGUSR2 as usize => {
                        self::send_signal(tx, Signal::SIGUSR2)
                    }
                    signal => {
                        return HandleResult::Stop(StopReason::Fatal(format!("caught unexpected \
                                                                             signal: {}",
                                                                            signal)),
                                                  None);
                    }
                }

                // clear out the signal so we don't sent it repeatedly
                CAUGHT.store(false, Ordering::SeqCst);
                SIGNAL.store(0 as usize, Ordering::SeqCst);
            }
        }
        HandleResult::NoReply(Some(TIMEOUT_MS))
    }
}

fn send_signal(tx: &ActorSender<Message>, signal: Signal) {
    actor::cast(tx, Message::Signal(signal)).unwrap();
}

fn set_signal_handlers() {
    unsafe {
        signal(Signal::SIGHUP as u32, handle_signal);
        signal(Signal::SIGINT as u32, handle_signal);
        signal(Signal::SIGQUIT as u32, handle_signal);
        signal(Signal::SIGALRM as u32, handle_signal);
        signal(Signal::SIGTERM as u32, handle_signal);
        signal(Signal::SIGUSR1 as u32, handle_signal);
        signal(Signal::SIGUSR2 as u32, handle_signal);
    }
}

/// send a Unix signal to a pid
pub fn send_signal_to_pid(pid: u32, sig: Signal) -> Result<()> {
    let s = sig as u32;
    debug!("sending signal {} to pid {}", s, pid);
    unsafe {
        let result = kill(pid as i32, s);
        match result {
            0 => Ok(()),
            _ => return Err(sup_error!(Error::SignalFailed)),
        }
    }
}
