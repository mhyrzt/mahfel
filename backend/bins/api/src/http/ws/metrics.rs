use prometheus::{IntCounter, IntCounterVec, IntGauge, Opts, Registry};

/// Why a WebSocket connection ended. Kept as a closed set so the disconnect
/// counter stays bounded-cardinality.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisconnectReason {
    ClientClose,
    HeartbeatTimeout,
    Backpressure,
    Unauthorized,
    Shutdown,
    ProtocolError,
    Error,
}

impl DisconnectReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClientClose => "client_close",
            Self::HeartbeatTimeout => "heartbeat_timeout",
            Self::Backpressure => "backpressure",
            Self::Unauthorized => "unauthorized",
            Self::Shutdown => "shutdown",
            Self::ProtocolError => "protocol_error",
            Self::Error => "error",
        }
    }
}

/// Gateway counters and gauges registered on the operations registry.
#[derive(Clone)]
pub struct WsMetrics {
    connections_active: IntGauge,
    connections_total: IntCounter,
    connections_rejected_total: IntCounter,
    disconnects_total: IntCounterVec,
    messages_received_total: IntCounter,
    messages_sent_total: IntCounter,
    send_drops_total: IntCounter,
    rtc_requests_total: IntCounterVec,
}

impl WsMetrics {
    pub fn register(registry: &Registry) -> Result<Self, prometheus::Error> {
        let connections_active = IntGauge::with_opts(Opts::new(
            "websocket_connections_active",
            "Current open WebSocket connections",
        ))?;
        let connections_total = IntCounter::with_opts(Opts::new(
            "websocket_connections_total",
            "Total accepted WebSocket connections",
        ))?;
        let connections_rejected_total = IntCounter::with_opts(Opts::new(
            "websocket_connections_rejected_total",
            "Total WebSocket upgrades rejected at the connection cap",
        ))?;
        let disconnects_total = IntCounterVec::new(
            Opts::new(
                "websocket_disconnects_total",
                "WebSocket disconnects by reason",
            ),
            &["reason"],
        )?;
        let messages_received_total = IntCounter::with_opts(Opts::new(
            "websocket_messages_received_total",
            "Total WebSocket messages received",
        ))?;
        let messages_sent_total = IntCounter::with_opts(Opts::new(
            "websocket_messages_sent_total",
            "Total WebSocket messages sent",
        ))?;
        let send_drops_total = IntCounter::with_opts(Opts::new(
            "websocket_send_drops_total",
            "Total outbound messages dropped because a connection queue was full",
        ))?;

        registry.register(Box::new(connections_active.clone()))?;
        registry.register(Box::new(connections_total.clone()))?;
        registry.register(Box::new(connections_rejected_total.clone()))?;
        registry.register(Box::new(disconnects_total.clone()))?;
        registry.register(Box::new(messages_received_total.clone()))?;
        registry.register(Box::new(messages_sent_total.clone()))?;
        registry.register(Box::new(send_drops_total.clone()))?;
        let rtc_requests_total = IntCounterVec::new(
            Opts::new(
                "websocket_rtc_requests_total",
                "RTC signaling requests by type and status",
            ),
            &["type", "status"],
        )?;
        registry.register(Box::new(rtc_requests_total.clone()))?;

        Ok(Self {
            connections_active,
            connections_total,
            connections_rejected_total,
            disconnects_total,
            messages_received_total,
            messages_sent_total,
            send_drops_total,
            rtc_requests_total,
        })
    }

    pub fn connection_opened(&self) {
        self.connections_active.inc();
        self.connections_total.inc();
    }

    pub fn connection_closed(&self) {
        self.connections_active.dec();
    }

    pub fn connection_rejected(&self) {
        self.connections_rejected_total.inc();
    }

    pub fn disconnected(&self, reason: DisconnectReason) {
        self.disconnects_total
            .with_label_values(&[reason.as_str()])
            .inc();
    }

    pub fn message_received(&self) {
        self.messages_received_total.inc();
    }

    pub fn message_sent(&self) {
        self.messages_sent_total.inc();
    }

    pub fn send_dropped(&self) {
        self.send_drops_total.inc();
    }

    pub fn rtc_request(&self, kind: &str, status: &str) {
        self.rtc_requests_total
            .with_label_values(&[kind, status])
            .inc();
    }
}
