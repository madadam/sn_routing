// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Command;
use crate::event::Event;
use bytes::Bytes;
use std::{
    net::SocketAddr,
    slice,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tokio::sync::mpsc;

/// Context passed to all the command handler methods. Used for pushing new commands to the command
/// queue and for sending user events.
pub(crate) struct Context {
    command_queue: Vec<Command>,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl Context {
    pub fn new(event_tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            command_queue: Vec::new(),
            event_tx,
        }
    }

    /// Push new command into the command queue. The queue is processed after the current command
    /// handling completes.
    pub fn push_command(&mut self, command: Command) {
        self.command_queue.push(command);
    }

    /// Convenience method for pushing `Command::SendMessage` with a single recipient.
    pub fn send_message_to_target(&mut self, recipient: &SocketAddr, message: Bytes) {
        self.send_message_to_targets(slice::from_ref(recipient), 1, message)
    }

    /// Convenience method for pushing `Command::SendMessage` with multiple recipients.
    pub fn send_message_to_targets(
        &mut self,
        recipients: &[SocketAddr],
        delivery_group_size: usize,
        message: Bytes,
    ) {
        self.push_command(Command::SendMessage {
            recipients: recipients.to_vec(),
            delivery_group_size,
            message,
        })
    }

    /// Convenience method for pushing `Command::ScheduleTimeout` with a new timer token.
    pub fn schedule_timeout(&mut self, duration: Duration) -> u64 {
        let token = NEXT_TIMER_TOKEN.fetch_add(1, Ordering::Relaxed);
        self.push_command(Command::ScheduleTimeout { duration, token });
        token
    }

    pub fn into_commands(self) -> Vec<Command> {
        self.command_queue
    }

    /// Send user event.
    pub fn send_event(&mut self, event: Event) {
        if self.event_tx.send(event).is_err() {
            error!("Event receiver has been closed");
        }
    }
}

static NEXT_TIMER_TOKEN: AtomicU64 = AtomicU64::new(0);
