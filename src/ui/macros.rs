#[macro_export]
macro_rules! stateless_action {
    ($group:ident, $name:ident, $fun:expr) => {
        $group.add_action(RelmAction::<$name>::new_stateless($fun));
    };
}
