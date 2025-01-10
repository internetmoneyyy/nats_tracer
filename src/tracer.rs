use async_nats::Connection as NatsClient;
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Debug, Serialize, Clone)]
pub struct LogMessage {
    pub message: String,
}

#[derive(Debug, Serialize, Clone)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::INFO => write!(f, "INFO"),
            LogLevel::WARN => write!(f, "WARN"),
            LogLevel::ERROR => write!(f, "ERROR"),
            LogLevel::DEBUG => write!(f, "DEBUG"),
        }
    }
}

static TRACER: OnceCell<Tracer> = OnceCell::const_new();

pub struct Tracer {
    nats_client: Arc<NatsClient>,
    topic: String,
}

impl Tracer {
    pub fn new(nats_client: Arc<NatsClient>, topic: String) -> Self {
        Self { nats_client, topic }
    }

    pub async fn init(nats_client: Arc<NatsClient>, topic: String) {
        if TRACER.set(Tracer::new(nats_client, topic)).is_err() {
            panic!("Global tracer was already initialized");
        }
    }

    pub fn global() -> &'static Tracer {
        TRACER.get().expect("Tracer not initialized")
    }

    pub async fn log(
        &self,
        level: LogLevel,
        message: &str,
        module: &str,
    ) -> Result<(), anyhow::Error> {
        let log = LogMessage {
            message: message.to_string(),
        };

        let formatted_message = format!("{}", log.message);

        self.nats_client
            .publish(&self.topic.clone(), formatted_message.into_bytes())
            .await?;

        Ok(())
    }
}
