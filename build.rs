fn main() {
    // 可选：告诉 Cargo 在 build.rs 改变时重新运行
    println!("cargo:rerun-if-changed=build.rs");

    // 尝试获取本地时间，失败则回退到 UTC 或默认值
    let build_time = get_build_time();

    // 将时间注入为环境变量，供主程序通过 env! 读取
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);
}

fn get_build_time() -> String {
    // 优先尝试本地时间
    if let Ok(local) = time::OffsetDateTime::now_local() {
        if let Ok(formatted) = local.format(
            &time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
                .unwrap(),
        ) {
            return formatted;
        }
    }

    // 回退到 UTC 时间
    let utc = time::OffsetDateTime::now_utc();
    if let Ok(formatted) = utc.format(
        &time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
            .unwrap(),
    ) {
        return format!("{} (UTC)", formatted);
    }

    // 最终回退
    "unknown-build-time".to_string()
}