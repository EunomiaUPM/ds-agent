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

//! The TCK harness: mapping lookups, the scenario queue, the worker with a recording player,
//! and the TCK as a peer.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::flow::FlowError;
use crate::tck::{
    TckMapping, TckPeer, TckPlayerTrait, TckQueue, TckScriptTrait, TckStep, TckWorker,
};
use ymir::boot::workers::BackgroundWorker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    Start,
    Stop,
}

#[derive(Debug, Clone, Copy)]
enum Event {
    Requested,
}

struct Script;

impl TckScriptTrait for Script {
    type Message = Message;
    type Event = Event;
    const SECTIONS: &'static [&'static str] = &["provider_agreements", "consumer_agreements"];

    fn script(test: &str, event: Event) -> &'static [TckStep<Message>] {
        match (test, event) {
            ("T1", Event::Requested) => &[
                TckStep::Send(Message::Start),
                TckStep::Pause,
                TckStep::Send(Message::Stop),
            ],
            _ => &[],
        }
    }
}

/// Records what it was asked to send, failing the first `fail` calls.
struct Recorder {
    sent: Mutex<Vec<(String, Message)>>,
    fail: Mutex<u32>,
}

#[async_trait::async_trait]
impl TckPlayerTrait<Message> for Recorder {
    async fn send(&self, pid: &str, message: Message) -> Result<(), FlowError> {
        let mut fail = self.fail.lock().unwrap();
        if *fail > 0 {
            *fail -= 1;
            return Err(FlowError::NotFound("peer not ready".into()));
        }
        self.sent.lock().unwrap().push((pid.to_string(), message));
        Ok(())
    }
}

fn mapping_file() -> PathBuf {
    std::env::temp_dir().join(format!("tck-mapping-{}.json", uuid::Uuid::new_v4()))
}

fn write_mapping(path: &PathBuf) {
    std::fs::write(
        path,
        r#"{"datasets": ["urn:dataset:1"], "provider_agreements": {"T1": "urn:a:1"},
            "consumer_agreements": {"T2": "urn:a:2"}}"#,
    )
    .unwrap();
}

/// A resource is found in any of the asked sections; other sections and shapes are ignored.
#[test]
fn mapping_finds_the_test_in_any_section() {
    let path = mapping_file();
    write_mapping(&path);
    let mapping = TckMapping::new(&path);
    let sections = Script::SECTIONS;
    assert_eq!(
        mapping.test_of(sections, "urn:a:1").unwrap().as_deref(),
        Some("T1")
    );
    assert_eq!(
        mapping.test_of(sections, "urn:a:2").unwrap().as_deref(),
        Some("T2")
    );
    assert_eq!(
        mapping
            .test_of(&["provider_agreements"], "urn:a:2")
            .unwrap(),
        None
    );
    assert_eq!(mapping.test_of(sections, "urn:dataset:1").unwrap(), None);
}

/// The file is read on each lookup: missing at first, then picked up once seeded.
#[test]
fn mapping_is_read_when_needed() {
    let path = mapping_file();
    let mapping = TckMapping::new(&path);
    assert!(mapping.test_of(Script::SECTIONS, "urn:a:1").is_err());
    write_mapping(&path);
    assert!(mapping
        .test_of(Script::SECTIONS, "urn:a:1")
        .unwrap()
        .is_some());
}

/// Only a resource with steps for the event queues a job.
#[test]
fn queue_only_takes_scenarios_with_steps() {
    let path = mapping_file();
    write_mapping(&path);
    let (queue, mut jobs) = TckQueue::<Script>::open(TckMapping::new(&path));
    queue.enqueue("urn:a:2", Event::Requested, "pid-2".into());
    queue.enqueue("urn:a:unknown", Event::Requested, "pid-3".into());
    queue.enqueue("urn:a:1", Event::Requested, "pid-1".into());
    let job = jobs.0.try_recv().unwrap();
    assert_eq!(job.pid, "pid-1");
    assert_eq!(job.steps.len(), 3);
    assert!(jobs.0.try_recv().is_err());
}

/// The worker plays every step in order on the job's pid, retrying a step the peer refused.
#[tokio::test(start_paused = true)]
async fn worker_plays_the_steps_with_retries() {
    let path = mapping_file();
    write_mapping(&path);
    let (queue, jobs) = TckQueue::<Script>::open(TckMapping::new(&path));
    let recorder = Arc::new(Recorder {
        sent: Mutex::new(Vec::new()),
        fail: Mutex::new(1),
    });
    let worker = Box::new(TckWorker::new("test-tck", jobs, recorder.clone()));
    let token = CancellationToken::new();
    let running = tokio::spawn(worker.run(token.clone()));
    queue.enqueue("urn:a:1", Event::Requested, "pid-1".into());
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    token.cancel();
    running.await.unwrap().unwrap();
    assert_eq!(
        *recorder.sent.lock().unwrap(),
        vec![
            ("pid-1".to_string(), Message::Start),
            ("pid-1".to_string(), Message::Stop)
        ]
    );
}

/// In TCK mode the TCK's id is its token, both on our routes and towards it; off, it is nothing.
#[test]
fn the_tck_id_is_its_token_only_in_tck_mode() {
    let peer = TckPeer::verify(true, "TCK_PARTICIPANT").unwrap();
    assert_eq!(peer.participant_id, TckPeer::PARTICIPANT_ID);
    assert!(TckPeer::verify(true, "another-token").is_none());
    assert!(TckPeer::verify(false, "TCK_PARTICIPANT").is_none());
    assert_eq!(
        TckPeer::token_for(true, "TCK_PARTICIPANT").as_deref(),
        Some("TCK_PARTICIPANT")
    );
    assert!(TckPeer::token_for(true, "did:web:peer").is_none());
    assert!(TckPeer::token_for(false, "TCK_PARTICIPANT").is_none());
}
