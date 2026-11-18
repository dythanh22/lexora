# Yêu cầu phi chức năng và cách đo

Mọi con số dưới đây là **mục tiêu khởi điểm**, chưa phải kết quả đo. Điền kết quả vào cột cuối cùng cùng đường dẫn tới bằng chứng trong `tests/evidence/`.

| NFR | Nội dung | Cách đo | Kết quả |
|---|---|---|---|
| NFR01 | Từ lúc kích hoạt đến khi cửa sổ hiện ≤ 300 ms (chưa tính mạng), p95 | 100 lần, warm start; ghi riêng cold start | Chưa đo |
| NFR02 | Ngân sách capture 700 ms/action; hết thì chuyển nhập tay | Đồng hồ monotonic trong `run_action` | Chưa đo |
| NFR04 | Tỷ lệ direct support trên nhóm ứng dụng "được hỗ trợ" | Xem lưu ý thống kê dưới đây | Chưa đo |
| NFR07 | Enter không gửi khi IME đang soạn chữ (tiếng Việt) | Kịch bản thủ công Telex/VNI trên cả hai nền tảng | Chưa đo |
| NFR08 | Timeout HTTP: connect 5 s, tổng 20 s | Test provider với server giả chậm | Chưa đo |
| RAM/dung lượng | Không đặt cam kết trước khi đo (WebKitGTK/WebView2) | POC-R | Chưa đo |

## Lưu ý thống kê (NFR04)

Ngưỡng "≥ 38/40" có cận dưới tin cậy 95% khoảng 83%, và "0 lần trả nhầm văn bản cũ" không thể được chứng minh bằng 40 lần thử. Dùng ngưỡng này như **bộ lọc sàng lọc**, hoặc tăng số mẫu cho nhóm direct support và báo cáo kèm khoảng tin cậy (ví dụ Wilson).
