//! Provider giả cho test hợp đồng (T06/T07). Không chứng minh tương thích với
//! dịch vụ thật (§11 POC-T).
use std::sync::Mutex;

use lexora_core::ports::{BoxFuture, EventSink, TranslationProvider};
use lexora_core::{
    AppError, AppErrorCode, AttemptId, AuthMode, ProviderCapabilities, ProviderEvent,
    ProviderEventKind, TranslationRequest,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub enum Step {
    Delta(String),
    Complete(String),
    Fail(AppErrorCode),
}

pub struct FakeProvider {
    pub script: Vec<Step>,
}

impl TranslationProvider for FakeProvider {
    fn id(&self) -> &'static str {
        "fake"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            auto_detect: true,
            streaming: true,
            max_input_chars: None,
            auth: AuthMode::None,
        }
    }

    fn translate<'a>(
        &'a self,
        request: &'a TranslationRequest,
        attempt: AttemptId,
        cancel: CancellationToken,
        sink: &'a dyn EventSink,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let mut seq = 0u32;
            let mut emit = |kind: ProviderEventKind| {
                sink.emit(ProviderEvent {
                    request_id: request.request_id,
                    attempt_id: attempt,
                    sequence: seq,
                    kind,
                });
                seq += 1;
            };
            emit(ProviderEventKind::Started);
            for step in &self.script {
                if cancel.is_cancelled() {
                    return;
                }
                match step {
                    Step::Delta(t) => emit(ProviderEventKind::Delta { text: t.clone() }),
                    Step::Complete(t) => {
                        emit(ProviderEventKind::Completed { text: t.clone(), detected: None })
                    }
                    Step::Fail(code) => emit(ProviderEventKind::Failed { error: AppError::new(*code) }),
                }
            }
        })
    }
}

/// Sink thu sự kiện để assert trong test.
#[derive(Default)]
pub struct CollectingSink {
    pub events: Mutex<Vec<ProviderEvent>>,
}

impl EventSink for CollectingSink {
    fn emit(&self, event: ProviderEvent) {
        if let Ok(mut v) = self.events.lock() {
            v.push(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexora_core::{LangCode, RequestId, SourceLang};

    fn req() -> TranslationRequest {
        TranslationRequest {
            request_id: RequestId(1),
            text: "hello".into(),
            source: SourceLang::Auto,
            target: LangCode::default(),
            config_version: 0,
        }
    }

    #[tokio::test]
    async fn emits_ordered_events_with_single_terminal() {
        let p = FakeProvider {
            script: vec![Step::Delta("xin".into()), Step::Complete("xin chào".into())],
        };
        let sink = CollectingSink::default();
        let r = req();
        p.translate(&r, AttemptId(1), CancellationToken::new(), &sink).await;
        let evs = sink.events.lock().unwrap();
        let seqs: Vec<u32> = evs.iter().map(|e| e.sequence).collect();
        assert_eq!(seqs, vec![0, 1, 2]);
        assert_eq!(evs.iter().filter(|e| e.is_terminal()).count(), 1);
    }

    #[tokio::test]
    async fn cancelled_provider_emits_no_terminal() {
        let p = FakeProvider { script: vec![Step::Complete("x".into())] };
        let sink = CollectingSink::default();
        let token = CancellationToken::new();
        token.cancel();
        let r = req();
        p.translate(&r, AttemptId(1), token, &sink).await;
        assert!(sink.events.lock().unwrap().iter().all(|e| !e.is_terminal()));
    }
}
