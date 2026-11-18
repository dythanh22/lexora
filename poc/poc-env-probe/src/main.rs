//! In ra thông tin môi trường để dán vào bằng chứng PoC. Chỉ dùng std, không phụ thuộc gì.
//! Các biến chỉ là GỢI Ý về phiên hiển thị (thiết kế §7.5); PoC phải xác nhận bằng cách khác.
use std::env;

fn show(key: &str) {
    match env::var(key) {
        Ok(v) if !v.is_empty() => println!("{key}={v}"),
        _ => println!("{key}=<unset>"),
    }
}

fn main() {
    println!("os={} arch={}", env::consts::OS, env::consts::ARCH);
    for key in [
        "XDG_SESSION_TYPE",
        "XDG_CURRENT_DESKTOP",
        "WAYLAND_DISPLAY",
        "DISPLAY",
        "GDK_BACKEND",
        "XDG_RUNTIME_DIR",
        "DESKTOP_SESSION",
    ] {
        show(key);
    }
    // Chỉ cho biết có đặt hay không, không in địa chỉ bus.
    let bus = env::var("DBUS_SESSION_BUS_ADDRESS").is_ok_and(|v| !v.is_empty());
    println!("DBUS_SESSION_BUS_ADDRESS={}", if bus { "<set>" } else { "<unset>" });
    println!("snap={} flatpak={}", env::var("SNAP").is_ok(), env::var("FLATPAK_ID").is_ok());
}
