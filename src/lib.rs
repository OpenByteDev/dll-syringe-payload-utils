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
        $crate::remote_procedure! {
            pub fn $fn ( $($name : $type),* ) -> () $body
        }
    };
    ($(pub)? fn $fn:ident ( $($name:ident : $type:ty),* ) -> $ret:ty 
        $body:block 
    ) => {
        #[no_mangle]
        pub extern "system" fn $fn ( __args: *const ($($type ,)*), __result: *mut $ret ) {
            fn __inner ( $($name : $type),* ) -> $ret $body

            let ($($name ,)*) = unsafe { ::core::ptr::read(__args) };
            unsafe { ::core::ptr::write(__result, __inner($($name ,)*)) };
        }
    };
}

#[cfg(test)]
mod tests {
    use std::{mem::MaybeUninit, ptr};

    #[test]
    fn does_not_drop_uninit_result() {
        pub struct NoImplicitDrop(u32);

        impl Drop for NoImplicitDrop {
            fn drop(&mut self) {
                assert_ne!(self.0, 0, "dropped before being initialized");
            }
        }

        impl NoImplicitDrop {
            fn new() -> Self {
                Self(1)
            }
        }

        remote_procedure! {
            fn add() -> NoImplicitDrop {
                NoImplicitDrop::new()
            }
        }

        let mut result = MaybeUninit::uninit();
        add(&(), result.as_mut_ptr());
    }

    #[test]
    fn works_with_byte_buf() {
        remote_procedure! {
            fn pass_bytes(_arg: &[u8]) {
            }
        }
    }

    #[test]
    fn takes_ownership_of_arg() {
        pub struct NoCopy(u32);

        impl Drop for NoCopy {
            fn drop(&mut self) {
            }
        }

        static mut STORAGE: Option<NoCopy> = None;

        remote_procedure! {
            fn takes_ownership(arg: NoCopy) {
                unsafe { STORAGE = Some(arg) }
            }
        }

        let mut arg = NoCopy(1);
        let arg_ptr = &mut arg as *mut NoCopy;
        takes_ownership(arg_ptr.cast(), &mut ());
        unsafe { ptr::write(arg_ptr, NoCopy(2)) };

        assert_eq!(unsafe { STORAGE.take() }.unwrap().0, 1);
    }
}
