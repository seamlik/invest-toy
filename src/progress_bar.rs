#[mockall::automock]
pub trait ProgressBar {
    fn set_length(&self, length: u64);
    fn advance(&self);
    fn finish(&self);
    fn abandon(&self);
}

impl ProgressBar for indicatif::ProgressBar {
    fn set_length(&self, length: u64) {
        self.set_length(length)
    }

    fn advance(&self) {
        self.inc(1)
    }

    fn finish(&self) {
        self.finish()
    }

    fn abandon(&self) {
        self.abandon()
    }
}
