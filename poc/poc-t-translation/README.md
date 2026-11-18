# POC-T: Dịch máy

## Cần ghi lại
- Azure Translator F0: thời gian từ cài đặt tới lần dịch đầu (time-to-first-translation), cặp EN→VI trên `tests/corpus/`, lỗi auth/quota/429, mã lỗi thật (403001?), header region.
- Hợp đồng stream bằng `FakeProvider`: hủy, sự kiện cũ, dữ liệu hỏng.

## Quyết định sau PoC
Azure có đủ nhẹ về onboarding và đạt chất lượng để làm engine MVP không (ADR-0004).
