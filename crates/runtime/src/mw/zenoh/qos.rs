// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use zenoh::qos::CongestionControl;
use zenoh::qos::Priority;
use zenoh::sample::Locality;

use crate::mw::{
    MessagePriority, PublishQos, Reliability, SubscribeQos, SubscriptionOrigin,
};

pub(crate) fn apply_publish_qos<'a, 'b>(
    qos: PublishQos,
    builder: zenoh::pubsub::PublisherBuilder<'a, 'b>,
) -> zenoh::pubsub::PublisherBuilder<'a, 'b> {
    builder
        .priority(priority(qos.priority))
        .congestion_control(congestion(qos.reliability))
        .express(qos.express)
}

pub(crate) fn apply_subscribe_qos<'a, 'b>(
    qos: SubscribeQos,
    builder: zenoh::pubsub::SubscriberBuilder<'a, 'b, zenoh::handlers::DefaultHandler>,
) -> zenoh::pubsub::SubscriberBuilder<'a, 'b, zenoh::handlers::DefaultHandler> {
    builder.allowed_origin(origin(qos.origin))
}

fn priority(value: MessagePriority) -> Priority {
    match value {
        MessagePriority::Low => Priority::Background,
        MessagePriority::Normal => Priority::Data,
        MessagePriority::High => Priority::RealTime,
    }
}

fn congestion(value: Reliability) -> CongestionControl {
    match value {
        Reliability::BestEffort => CongestionControl::Drop,
        Reliability::Reliable => CongestionControl::Block,
    }
}

fn origin(value: SubscriptionOrigin) -> Locality {
    match value {
        SubscriptionOrigin::SessionLocal => Locality::SessionLocal,
        SubscriptionOrigin::Remote => Locality::Remote,
        SubscriptionOrigin::Any => Locality::Any,
    }
}
