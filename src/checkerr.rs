#[macro_export]
macro_rules! checkerr {
    ($ex:expr) => {
        if $ex.is_err() {
            return Err(());
        }
    };
}
