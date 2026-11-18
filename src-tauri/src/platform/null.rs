//! Reader rỗng: nghĩa là "chưa hỗ trợ", KHÔNG phải lỗi, và KHÔNG được rơi về clipboard.
use std::time::Duration;

use lexora_core::ports::{BoxFuture, ClipboardReader, SelectionReader};
use lexora_core::{AcquiredText, AppError, TriggerContext};

pub struct NullSelectionReader;

impl SelectionReader for NullSelectionReader {
    fn backend_id(&self) -> &'static str {
        "null"
    }

    fn read_selection<'a>(
        &'a self,
        _ctx: &'a TriggerContext,
        _deadline: Duration,
    ) -> BoxFuture<'a, Result<Option<AcquiredText>, AppError>> {
        Box::pin(async { Ok(None) })
    }
}

pub struct NullClipboardReader;

impl ClipboardReader for NullClipboardReader {
    fn backend_id(&self) -> &'static str {
        "null"
    }

    fn read_text(&self) -> BoxFuture<'_, Result<Option<String>, AppError>> {
        Box::pin(async { Ok(None) })
    }
}
