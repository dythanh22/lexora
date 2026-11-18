#![allow(dead_code)]
//! Chỉ dành cho X11 (Ubuntu 24.04 và các desktop X11 khác).
//!
//! TODO(POC-S): PRIMARY selection — chỉ dùng khi chứng minh còn là lựa chọn hiện tại của
//!   nguồn; owner có thể là helper/proxy, không quy chiếu được thì coi là không đáng tin (§7.2).
//! TODO(POC-C): Copy giả lập (XTest) có snapshot/restore clipboard; tùy chọn, chỉ cho ứng dụng đã test.
//! TODO(POC-H): `XGrabKey` qua plugin global-shortcut.
