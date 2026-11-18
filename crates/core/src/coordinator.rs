//! Máy trạng thái điều phối phiên dịch (§8.4, FR07, BR03–BR08).
//!
//! Cố ý là cấu trúc ĐỒNG BỘ, không I/O, không phụ thuộc runtime: toàn bộ chính sách
//! chống race (request/attempt/sequence) test được bằng unit test thuần.
//! Shell bọc nó trong `Mutex` và không giữ khóa qua `.await`.
use serde::Serialize;

use crate::error::{AppError, AppErrorCode};
use crate::language::validate_source_text;
use crate::model::{Action, AttemptId, ProviderEvent, ProviderEventKind, RequestId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Acquiring,
    NeedsInput,
    Editing,
    Translating,
    Result,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitOutcome {
    Started { attempt_id: AttemptId },
    /// BR08: cùng nguồn đang chạy thì bỏ trigger lặp (key-repeat).
    DuplicateIgnored,
}

#[derive(Debug)]
struct Current {
    request_id: RequestId,
    attempt_id: AttemptId,
    source_text: Option<String>,
    last_sequence: Option<u32>,
}

#[derive(Debug)]
pub struct Coordinator {
    phase: Phase,
    next_request: u64,
    current: Option<Current>,
}

impl Default for Coordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Coordinator {
    pub fn new() -> Self {
        Self { phase: Phase::Idle, next_request: 1, current: None }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn current_request(&self) -> Option<RequestId> {
        self.current.as_ref().map(|c| c.request_id)
    }

    fn is_current(&self, id: RequestId) -> bool {
        self.current.as_ref().is_some_and(|c| c.request_id == id)
    }

    /// Mọi action mới tạo request mới và vô hiệu hóa phiên trước.
    pub fn begin(&mut self, action: Action) -> RequestId {
        let id = RequestId(self.next_request);
        self.next_request += 1;
        self.phase = match action {
            Action::OpenInput => Phase::Editing,
            Action::TranslateSelection | Action::TranslateClipboard => Phase::Acquiring,
        };
        self.current = Some(Current {
            request_id: id,
            attempt_id: AttemptId(0),
            source_text: None,
            last_sequence: None,
        });
        id
    }

    /// Capture thất bại/không hỗ trợ: chuyển sang hướng dẫn, không gửi gì (BR03).
    pub fn capture_failed(&mut self, id: RequestId) -> bool {
        if self.is_current(id) && self.phase == Phase::Acquiring {
            self.phase = Phase::NeedsInput;
            true
        } else {
            false
        }
    }

    /// Gửi nguồn (từ capture hoặc người dùng nhập) để bắt đầu một attempt mới.
    pub fn submit(
        &mut self,
        id: RequestId,
        text: &str,
        max_chars: usize,
    ) -> Result<SubmitOutcome, AppError> {
        if !self.is_current(id) {
            return Err(AppError::new(AppErrorCode::Cancelled));
        }
        validate_source_text(text, max_chars)?;

        let translating = self.phase == Phase::Translating;
        let Some(cur) = self.current.as_mut() else {
            return Err(AppError::new(AppErrorCode::Cancelled));
        };
        if translating && cur.source_text.as_deref() == Some(text) {
            return Ok(SubmitOutcome::DuplicateIgnored);
        }
        cur.attempt_id = AttemptId(cur.attempt_id.0 + 1);
        cur.source_text = Some(text.to_owned());
        cur.last_sequence = None;
        let attempt_id = cur.attempt_id;
        self.phase = Phase::Translating;
        Ok(SubmitOutcome::Started { attempt_id })
    }

    /// Retry là attempt mới do người dùng bấm; có thể tốn thêm quota (BR04).
    pub fn retry(&mut self, id: RequestId) -> Result<AttemptId, AppError> {
        if !self.is_current(id) || self.phase != Phase::Error {
            return Err(AppError::new(AppErrorCode::Cancelled));
        }
        let Some(cur) = self.current.as_mut() else {
            return Err(AppError::new(AppErrorCode::Cancelled));
        };
        if cur.source_text.is_none() {
            return Err(AppError::new(AppErrorCode::InputInvalid));
        }
        cur.attempt_id = AttemptId(cur.attempt_id.0 + 1);
        cur.last_sequence = None;
        let attempt_id = cur.attempt_id;
        self.phase = Phase::Translating;
        Ok(attempt_id)
    }

    /// Sửa nguồn sau khi có kết quả hoặc lỗi; không tự gọi API (§4.4).
    pub fn edit_source(&mut self) -> bool {
        if matches!(self.phase, Phase::Result | Phase::Error) {
            self.phase = Phase::Editing;
            true
        } else {
            false
        }
    }

    /// Hủy hoặc đóng: bỏ phiên, mọi sự kiện đến muộn đều bị loại (BR05).
    pub fn close(&mut self) {
        self.phase = Phase::Idle;
        self.current = None;
    }

    /// Trả `true` nếu sự kiện hợp lệ và cần chuyển tiếp cho UI.
    /// Loại: sai request/attempt, không ở Translating, sequence không tăng, đã kết thúc.
    pub fn on_event(&mut self, event: &ProviderEvent) -> bool {
        if self.phase != Phase::Translating {
            return false;
        }
        let Some(cur) = self.current.as_mut() else {
            return false;
        };
        if event.request_id != cur.request_id || event.attempt_id != cur.attempt_id {
            return false;
        }
        if let Some(last) = cur.last_sequence {
            if event.sequence <= last {
                return false;
            }
        }
        cur.last_sequence = Some(event.sequence);
        match event.kind {
            ProviderEventKind::Completed { .. } => self.phase = Phase::Result,
            ProviderEventKind::Failed { .. } => self.phase = Phase::Error,
            ProviderEventKind::Started | ProviderEventKind::Delta { .. } => {}
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::MAX_INPUT_CHARS;

    fn ev(req: RequestId, att: AttemptId, seq: u32, kind: ProviderEventKind) -> ProviderEvent {
        ProviderEvent { request_id: req, attempt_id: att, sequence: seq, kind }
    }

    fn done(text: &str) -> ProviderEventKind {
        ProviderEventKind::Completed { text: text.into(), detected: None }
    }

    fn fail() -> ProviderEventKind {
        ProviderEventKind::Failed { error: AppError::new(AppErrorCode::Network) }
    }

    fn started(c: &mut Coordinator, id: RequestId, text: &str) -> AttemptId {
        match c.submit(id, text, MAX_INPUT_CHARS).unwrap() {
            SubmitOutcome::Started { attempt_id } => attempt_id,
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn happy_path_reaches_result() {
        let mut c = Coordinator::new();
        let r = c.begin(Action::TranslateSelection);
        assert_eq!(c.phase(), Phase::Acquiring);
        let a = started(&mut c, r, "hello");
        assert_eq!(c.phase(), Phase::Translating);
        assert!(c.on_event(&ev(r, a, 0, ProviderEventKind::Started)));
        assert!(c.on_event(&ev(r, a, 1, done("xin chào"))));
        assert_eq!(c.phase(), Phase::Result);
    }

    #[test]
    fn new_request_invalidates_old_events() {
        let mut c = Coordinator::new();
        let r1 = c.begin(Action::TranslateSelection);
        let a1 = started(&mut c, r1, "A");
        let r2 = c.begin(Action::TranslateClipboard);
        assert!(!c.on_event(&ev(r1, a1, 0, done("late A"))));
        assert_ne!(r1, r2);
        assert_eq!(c.phase(), Phase::Acquiring);
    }

    #[test]
    fn retry_makes_new_attempt_and_drops_old_stream() {
        let mut c = Coordinator::new();
        let r = c.begin(Action::OpenInput);
        let a1 = started(&mut c, r, "hello");
        assert!(c.on_event(&ev(r, a1, 0, fail())));
        assert_eq!(c.phase(), Phase::Error);
        let a2 = c.retry(r).unwrap();
        assert_eq!(a2, AttemptId(a1.0 + 1));
        assert!(!c.on_event(&ev(r, a1, 5, done("stale"))));
        assert!(c.on_event(&ev(r, a2, 0, done("ok"))));
        assert_eq!(c.phase(), Phase::Result);
    }

    #[test]
    fn out_of_order_and_post_terminal_events_are_dropped() {
        let mut c = Coordinator::new();
        let r = c.begin(Action::OpenInput);
        let a = started(&mut c, r, "hello");
        let delta = |s: &str| ProviderEventKind::Delta { text: s.into() };
        assert!(c.on_event(&ev(r, a, 0, ProviderEventKind::Started)));
        assert!(c.on_event(&ev(r, a, 2, delta("x"))));
        assert!(!c.on_event(&ev(r, a, 1, delta("y"))));
        assert!(c.on_event(&ev(r, a, 3, done("z"))));
        assert!(!c.on_event(&ev(r, a, 4, fail())));
        assert_eq!(c.phase(), Phase::Result);
    }

    #[test]
    fn duplicate_trigger_ignored_only_while_translating() {
        let mut c = Coordinator::new();
        let r = c.begin(Action::TranslateSelection);
        let a = started(&mut c, r, "same");
        assert_eq!(
            c.submit(r, "same", MAX_INPUT_CHARS).unwrap(),
            SubmitOutcome::DuplicateIgnored
        );
        assert!(c.on_event(&ev(r, a, 0, done("xong"))));
        // Sau khi hoàn tất, người dùng chủ động dịch lại được (BR08).
        assert!(matches!(
            c.submit(r, "same", MAX_INPUT_CHARS).unwrap(),
            SubmitOutcome::Started { .. }
        ));
    }

    #[test]
    fn close_drops_everything() {
        let mut c = Coordinator::new();
        let r = c.begin(Action::OpenInput);
        let a = started(&mut c, r, "hello");
        c.close();
        assert_eq!(c.phase(), Phase::Idle);
        assert!(!c.on_event(&ev(r, a, 0, done("late"))));
        let err = c.submit(r, "again", MAX_INPUT_CHARS).unwrap_err();
        assert_eq!(err.code, AppErrorCode::Cancelled);
    }

    #[test]
    fn blank_source_is_rejected_without_state_change() {
        let mut c = Coordinator::new();
        let r = c.begin(Action::TranslateSelection);
        let err = c.submit(r, "   ", MAX_INPUT_CHARS).unwrap_err();
        assert_eq!(err.code, AppErrorCode::InputInvalid);
        assert_eq!(c.phase(), Phase::Acquiring);
    }

    #[test]
    fn capture_failure_needs_input_and_never_translates() {
        let mut c = Coordinator::new();
        let r = c.begin(Action::TranslateSelection);
        assert!(c.capture_failed(r));
        assert_eq!(c.phase(), Phase::NeedsInput);
        // Người dùng dán/nhập tay rồi mới dịch.
        let _ = started(&mut c, r, "pasted");
        assert_eq!(c.phase(), Phase::Translating);
    }

    #[test]
    fn edit_source_only_after_result_or_error() {
        let mut c = Coordinator::new();
        let r = c.begin(Action::OpenInput);
        assert!(!c.edit_source());
        let a = started(&mut c, r, "hello");
        assert!(c.on_event(&ev(r, a, 0, done("ok"))));
        assert!(c.edit_source());
        assert_eq!(c.phase(), Phase::Editing);
    }
}
