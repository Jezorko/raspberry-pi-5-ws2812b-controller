#[macro_export]
macro_rules! tests {
    ($module_name:ident,$test_code:expr,$($test_case_name:ident:$test_case_parameters:expr,)*) => {
        #[cfg(test)]
        mod $module_name {
            #[allow(unused)]
            use pretty_assertions::{assert_eq};
            #[allow(unused)]
            use super::*;
            $(
                #[test]
                #[allow(non_snake_case)]
                fn $test_case_name() {
                    $test_code($test_case_parameters)
                }
            )*
        }
    }
}