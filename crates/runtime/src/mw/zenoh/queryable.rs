// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;

use crate::mw::{IncomingQuery, MwError, QueryReply, Queryable};

use super::session::ZenohSession;

struct ZenohQueryReply {
    query: zenoh::query::Query,
}

#[async_trait]
impl QueryReply for ZenohQueryReply {
    async fn respond(&self, payload: Vec<u8>) -> Result<(), MwError> {
        self.query
            .reply(self.query.key_expr(), payload)
            .await
            .map_err(|e| MwError::from(e.to_string()))
    }
}

pub(crate) struct ZenohQueryable {
    rx: tokio::sync::mpsc::UnboundedReceiver<IncomingQuery>,
}

impl ZenohQueryable {
    pub(crate) async fn declare(session: &ZenohSession, topic: String) -> Result<Self, MwError> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        session
            .inner()
            .declare_queryable(&topic)
            .complete(true)
            .callback(move |query| {
                let payload = query
                    .payload()
                    .map(|p| p.to_bytes().to_vec())
                    .unwrap_or_default();
                let reply = Box::new(ZenohQueryReply { query });
                let _ = tx.send(IncomingQuery::new(payload, reply));
            })
            .background()
            .await
            .map_err(|e| MwError::from(e.to_string()))?;

        Ok(Self { rx })
    }
}

#[async_trait]
impl Queryable for ZenohQueryable {
    async fn recv(&mut self) -> Result<Option<IncomingQuery>, MwError> {
        match self.rx.recv().await {
            Some(query) => Ok(Some(query)),
            None => Ok(None),
        }
    }
}
