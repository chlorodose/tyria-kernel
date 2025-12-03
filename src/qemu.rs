use log::Log;
use qemu_print::qemu_println;

#[derive(Debug, Clone)]
struct QemuLog;
impl Log for QemuLog {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn flush(&self) {}
    fn log(&self, record: &log::Record) {
        qemu_println!(
            "[{}]({}) {}",
            record.level(),
            record.target(),
            record.args()
        );
    }
}
static QEMU_LOG: QemuLog = QemuLog;
pub fn set_qemu_log() {
    let _ = log::set_logger(&QEMU_LOG);
    log::set_max_level(log::LevelFilter::Trace);
}
