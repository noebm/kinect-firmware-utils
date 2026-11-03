use std::io::Write;

pub(super) fn init() {
    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));

    if stderr_is_journal() {
        builder.format(|buffer, record| {
            let priority = match record.level() {
                log::Level::Error => 3,
                log::Level::Warn => 4,
                log::Level::Info => 6,
                log::Level::Debug | log::Level::Trace => 7,
            };

            writeln!(buffer, "<{priority}>{}", record.args())
        });
    }

    builder.init();
}

#[cfg(unix)]
fn stderr_is_journal() -> bool {
    use std::os::{fd::AsFd, unix::fs::MetadataExt};

    let Ok(stream) = std::env::var("JOURNAL_STREAM") else {
        return false;
    };
    let Ok(fd) = std::io::stderr().as_fd().try_clone_to_owned() else {
        return false;
    };
    let Ok(metadata) = std::fs::File::from(fd).metadata() else {
        return false;
    };

    stream == format!("{}:{}", metadata.dev(), metadata.ino())
}

#[cfg(not(unix))]
fn stderr_is_journal() -> bool {
    false
}
