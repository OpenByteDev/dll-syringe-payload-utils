//! Utilities for building payloads for [`dll-syringe`](https://docs.rs/dll-syringe/latest/dll_syringe/).

/// A macro for defining functions to be called using [`RemoteProcedure`](https://docs.rs/dll-syringe/latest/dll_syringe/struct.RemoteProcedure.html) from [`dll-syringe`](https://docs.rs/dll-syringe/latest/dll_syringe/)
/// in a more ergonomic manner.
/// 
/// # Example
/// ```
/// dll_syringe_payload_utils::remote_procedure! {
///     fn add(a: i32, b: i32) -> i32 {
///         a + b
///     }
/// }
/// #
/// # let args = (1, 2);
/// # let mut result = 0;
/// # add(&args, &mut result);
/// # assert_eq!(result, 3);
/// ```
/// 
/// will generate an equivalent function to
/// 
/// ```
/// #[no_mangle]
/// pub extern "system" fn add(args: *const (i32, i32), result: *mut i32) {
///     unsafe { *result = (*args).0 + (*args).1 }
/// }
/// #
/// # let args = (1, 2);
/// # let mut result = 0;
/// # add(&args, &mut result);
/// # assert_eq!(result, 3);
/// ```
#[macro_export]
macro_rules! remote_procedure {
    ($(pub)? fn $fn:ident ( $($name:ident : $type:ty),* )
        $body:block 
    ) => {
        pub $crate::remote_procedure! {
            fn $fn ( $($name : $type),* ) -> () $body
        }
    };
    ($(pub)? fn $fn:ident ( $($name:ident : $type:ty),* ) -> $ret:ty 
        $body:block 
    ) => {
        #[no_mangle]
        pub extern "system" fn $fn ( __args: *const ($($type ,)*), __result: *mut $ret ) {
            fn __inner ( $($name : $type),* ) -> $ret $body

            let ($($name ,)*) = unsafe { *__args };
            unsafe {
                *__result = __inner($($name ,)*);
            }
        }
    };
}
