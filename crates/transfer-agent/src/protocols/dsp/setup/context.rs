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

//! What the DSP plane uses: the agent context, the runners of both flows built on it, the
//! auto-start worker, and the TCK queue in TCK mode.

use std::sync::Arc;

use common::boot::workers::BackgroundWorker;
use common::flow::{AutocommitUnitOfWorkFactory, FlowRunner};
use common::flow::{IdempotencyGuard, InMemoryIdempotencyStore};
use common::tck::{TckMapping, TckQueue, TckWorker};

use crate::protocols::dsp::services::auto_start::guard::AutoStartGuard;
use crate::protocols::dsp::services::auto_start::worker::AutoStartWorker;
use crate::protocols::dsp::services::dsp_flow::service::TransferDspFlow;
use crate::protocols::dsp::services::dsp_persistence::service::TransferDspPersistence;
use crate::protocols::dsp::services::rpc_flow::service::TransferRpcFlow;
use crate::protocols::dsp::services::tck::guard::TckGuard;
use crate::protocols::dsp::services::tck::player::TransferTckPlayer;
use crate::protocols::dsp::services::tck::service::TransferTckScript;
use crate::protocols::dsp::services::validator::{TransferDspRules, TransferRpcRules};
use crate::setup::context::AppContext;

/// The agent context plus the runners peers and users reach through the DSP routes.
pub struct DspContext {
    pub app: Arc<AppContext>,
    pub persistence: TransferDspPersistence,
    pub inbound: Arc<FlowRunner<TransferDspFlow>>,
    pub outbound: Arc<FlowRunner<TransferRpcFlow>>,
    /// Set when `DSP_TCK_MAPPING` is.
    pub tck: Option<TckQueue<TransferTckScript>>,
}

impl DspContext {
    /// The runners and their workers: auto-start always, the TCK guard and player in TCK mode.
    pub fn build(app: Arc<AppContext>) -> (Arc<Self>, Vec<Box<dyn BackgroundWorker>>) {
        let (queue, jobs) = TckMapping::from_env()
            .map(TckQueue::<TransferTckScript>::open)
            .unzip();
        let guard = queue.clone().map(TckGuard::new);
        let (auto_start, auto_start_jobs) = AutoStartGuard::open();
        let outbound = Self::outbound(&app, guard.as_ref());
        let inbound = Self::inbound(&app, auto_start, guard.as_ref());
        let mut workers: Vec<Box<dyn BackgroundWorker>> =
            vec![Box::new(AutoStartWorker::new(auto_start_jobs, outbound.clone()))];
        if let Some(jobs) = jobs {
            let player = Arc::new(TransferTckPlayer::new(
                outbound.clone(),
                TransferDspPersistence::new(app.clone()),
            ));
            workers.push(Box::new(TckWorker::new("transfer-tck-scenarios", jobs, player)));
        }
        let ctx = Self {
            persistence: TransferDspPersistence::new(app.clone()),
            app,
            inbound,
            outbound,
            tck: queue,
        };
        (Arc::new(ctx), workers)
    }

    pub fn tck_mode(&self) -> bool {
        self.tck.is_some()
    }

    /// Peers' messages: replayed when repeated, then seen by the auto-start and TCK guards.
    fn inbound(
        app: &Arc<AppContext>,
        auto_start: AutoStartGuard,
        tck: Option<&TckGuard>,
    ) -> Arc<FlowRunner<TransferDspFlow>> {
        let mut runner = FlowRunner::builder(
            Arc::new(TransferDspFlow::new(app.clone())),
            Arc::new(AutocommitUnitOfWorkFactory),
            Arc::new(TransferDspRules::input_rules()),
            Arc::new(TransferDspRules::fact_rules()),
        )
        .guard(Arc::new(IdempotencyGuard::new(Arc::new(
            InMemoryIdempotencyStore::new(),
        ))))
        .guard(Arc::new(auto_start));
        if let Some(guard) = tck {
            runner = runner.guard(Arc::new(guard.clone()));
        }
        Arc::new(runner.build())
    }

    /// Our own messages, with the peer's answer checked.
    fn outbound(app: &Arc<AppContext>, tck: Option<&TckGuard>) -> Arc<FlowRunner<TransferRpcFlow>> {
        let mut runner = FlowRunner::builder(
            Arc::new(TransferRpcFlow::new(app.clone())),
            Arc::new(AutocommitUnitOfWorkFactory),
            Arc::new(TransferRpcRules::input_rules()),
            Arc::new(TransferRpcRules::fact_rules()),
        )
        .reply_rules(Arc::new(TransferRpcRules::reply_rules()));
        if let Some(guard) = tck {
            runner = runner.guard(Arc::new(guard.clone()));
        }
        Arc::new(runner.build())
    }
}
