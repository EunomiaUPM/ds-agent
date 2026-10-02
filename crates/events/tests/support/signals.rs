/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Waiting for the calls the bus makes from its background tasks.

use std::time::Duration;

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Channel a mock closure sends a label on when it is called.
pub struct Signals {
    pub tx: UnboundedSender<&'static str>,
    rx: UnboundedReceiver<&'static str>,
}

impl Signals {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        Self { tx, rx }
    }

    /// Waits up to five seconds for the next `n` labels, in arrival order.
    pub async fn next(&mut self, n: usize) -> Vec<&'static str> {
        let mut got = Vec::with_capacity(n);
        for _ in 0..n {
            let label = tokio::time::timeout(Duration::from_secs(5), self.rx.recv())
                .await
                .expect("background call within five seconds")
                .expect("signal channel open");
            got.push(label);
        }
        got
    }
}
