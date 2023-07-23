#[macro_export]
macro_rules! withclones {
    ($($var:ident),+) => {
        $(let $var = $var.clone();)+
    };
}

#[macro_export]
macro_rules! stateless_action {
    ($group:ident, $name:ident, $ex:expr) => {
        $group.add_action(RelmAction::<$name>::new_stateless(move |_| $ex));
    };
}
