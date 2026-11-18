//! AT-SPI qua accessibility bus (D-Bus): đường direct capture chính trên Wayland (§7.2).
//! Không phụ thuộc giao thức hiển thị, nhưng phụ thuộc toolkit/ứng dụng có phơi bày cây
//! accessibility hay không.
//!
//! TODO(POC-S):
//!  - Tìm phần tử đang focus, gọi `Text.get_selection`; deadline rõ ràng trên D-Bus.
//!  - Thử có/không bật cờ accessibility của phiên (`org.a11y.Status`) và ghi lại tác động.
//!  - Ghi kết quả theo từng (ứng dụng, gói .deb/Snap/Flatpak, native/XWayland) vào
//!    docs/support-matrix.md; không gộp mẫu để che lỗi.
use std::time::Duration;

use lexora_core::ports::{BoxFuture, SelectionReader};
use lexora_core::{AcquiredText, AppError, TriggerContext};

pub struct AtSpiSelectionReader;

impl SelectionReader for AtSpiSelectionReader {
    fn backend_id(&self) -> &'static str {
        "atspi"
    }

    fn read_selection<'a>(
        &'a self,
        _ctx: &'a TriggerContext,
        _deadline: Duration,
    ) -> BoxFuture<'a, Result<Option<AcquiredText>, AppError>> {
        // Chưa cài đặt: "không có vùng chọn" -> luồng hướng dẫn Copy (BR03).
        Box::pin(async { Ok(None) })
    }
}
