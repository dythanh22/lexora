# Corpus chất lượng EN→VI (T11, POC-T)

Mục tiêu theo thiết kế: bộ khoảng 100 đoạn công khai, người đọc chấm điểm 1–5; engine đạt khi ≥ 90% mẫu có điểm ≥ 4 (ngưỡng xem lại ở POC-T).

Quy tắc:
- Chỉ dùng văn bản có quyền sử dụng rõ ràng (tự viết, hoặc giấy phép cho phép). Ghi `license` và `source_note` cho từng dòng.
- Không đưa nội dung cá nhân hoặc bí mật vào corpus.
- Phân bố tối thiểu: giao diện/hướng dẫn, kỹ thuật, văn bản thông thường, câu có số liệu/định dạng, đoạn nhiều dòng.
- Mỗi lần chạy lưu điểm và ghi chú vào `tests/evidence/poc-t/`, không sửa corpus để "đạt".

Định dạng JSONL (`en-vi.sample.jsonl` là ví dụ do dự án tự viết):
`{"id": "...", "domain": "ui|technical|general|numeric|multiline", "text": "...", "license": "...", "source_note": "..."}`
