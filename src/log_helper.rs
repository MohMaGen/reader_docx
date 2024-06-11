pub trait LogHelper {
    fn log_if_error(&self);
}

impl LogHelper for anyhow::Result<()> {
    fn log_if_error(&self) {
        if let Err(err) = self {
            log::error!("{:?}", err);
        }
    }
}
