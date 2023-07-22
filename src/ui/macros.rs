#[macro_export]
macro_rules! withclones {
    ($($var:ident),+) => {
        $(let $var = $var.clone();)+
    };
}
