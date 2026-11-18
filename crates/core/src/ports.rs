//! Hợp đồng (ports) giữa core và thế giới bên ngoài (§8.1, §8.3).
//! Trait trả `BoxFuture` để dùng được dưới dạng `dyn` mà không cần crate bổ sung.
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::error::{AppError, AppErrorCode};
use crate::model::{
    AcquiredText, AttemptId, ProviderCapabilities, ProviderEvent, TranslationRequest,
    TriggerContext,
};
use crate::secret::{Secret, SecretRef};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Lấy văn bản đang chọn. `Ok(None)` nghĩa là "không có/không hỗ trợ", KHÔNG được
/// âm thầm rơi về clipboard (BR03, ADR-0003).
pub trait SelectionReader: Send + Sync {
    fn backend_id(&self) -> &'static str;
    fn read_selection<'a>(
        &'a self,
        ctx: &'a TriggerContext,
        deadline: Duration,
    ) -> BoxFuture<'a, Result<Option<AcquiredText>, AppError>>;
}

/// Đọc clipboard theo yêu cầu chủ động của người dùng. Không polling.
pub trait ClipboardReader: Send + Sync {
    fn backend_id(&self) -> &'static str;
    fn read_text(&self) -> BoxFuture<'_, Result<Option<String>, AppError>>;
}

/// Nơi provider đẩy sự kiện. Coordinator và UI đều lọc sự kiện cũ.
pub trait EventSink: Send + Sync {
    fn emit(&self, event: ProviderEvent);
}

/// Hợp đồng engine dịch.
///
/// Provider PHẢI phát đúng một sự kiện kết thúc (`Completed` hoặc `Failed`),
/// trừ khi `cancel` đã bị kích hoạt (khi đó có thể không phát gì).
/// Provider không streaming phát một `Completed`, không tạo token giả.
pub trait TranslationProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn capabilities(&self) -> ProviderCapabilities;
    fn translate<'a>(
        &'a self,
        request: &'a TranslationRequest,
        attempt: AttemptId,
        cancel: CancellationToken,
        sink: &'a dyn EventSink,
    ) -> BoxFuture<'a, ()>;
}

/// Chỉ phục vụ providers: lấy key cho một request, không bao giờ trả về UI.
pub trait SecretResolver: Send + Sync {
    fn resolve(&self, reference: &SecretRef) -> Result<Secret, AppError>;
}

/// Kho khóa. Cài đặt thật (Credential Manager / Secret Service) nằm ở shell.
/// Không có phương thức nào trả plaintext về UI.
pub trait SecretStore: Send + Sync {
    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, AppError>;
    fn set(&self, reference: &SecretRef, value: &str) -> Result<(), AppError>;
    fn delete(&self, reference: &SecretRef) -> Result<(), AppError>;
}

/// Tiện ích: biến `get` thành `resolve` với lỗi thiếu cấu hình chuẩn.
pub fn resolve_from_store(
    store: &dyn SecretStore,
    reference: &SecretRef,
) -> Result<Secret, AppError> {
    store
        .get(reference)?
        .ok_or_else(|| AppError::new(AppErrorCode::ConfigMissing))
}
