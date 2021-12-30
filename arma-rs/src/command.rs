use crate::arma::{ArmaValue, FromArma, IntoArma};
use crate::Context;

type HandlerFunc =
    Box<dyn Fn(Context, *mut libc::c_char, libc::c_int, Option<*mut *mut i8>, Option<libc::c_int>) -> libc::c_int>;

pub struct CommandHandler {
    pub(crate) handler: HandlerFunc,
}

pub fn fn_handler<C, I, R>(command: C) -> CommandHandler
where
    C: CommandFactory<I, R> + 'static,
{
    CommandHandler {
        handler: Box::new(
            move |context: Context,
                  output: *mut libc::c_char,
                  size: libc::c_int,
                  args: Option<*mut *mut i8>,
                  count: Option<libc::c_int>|
                  -> libc::c_int {
                unsafe { command.call(context, output, size, args, count) }
            },
        ),
    }
}

pub trait CommandExecutor: 'static {
    /// # Safety
    /// This function is unsafe because it interacts with the C API.
    unsafe fn call(
        &self,
        context: Context,
        output: *mut libc::c_char,
        size: libc::c_int,
        args: Option<*mut *mut i8>,
        count: Option<libc::c_int>,
    );
}

pub trait CommandFactory<A, R> {
    /// # Safety
    /// This function is unsafe because it interacts with the C API.
    unsafe fn call(
        &self,
        context: Context,
        output: *mut libc::c_char,
        size: libc::c_int,
        args: Option<*mut *mut i8>,
        count: Option<libc::c_int>,
    ) -> libc::c_int;
}

macro_rules! factory_tuple ({ $c: expr, $($param:ident)* } => {
    impl<$($param,)* O> CommandExecutor for dyn CommandFactory<($($param,)*), O>
    where
        O: 'static,
        $($param: FromArma + 'static,)*
    {
        unsafe fn call(
            &self,
            context: Context,
            output: *mut libc::c_char,
            size: libc::c_int,
            args: Option<*mut *mut i8>,
            count: Option<libc::c_int>,
        ) {
            self.call(context, output, size, args, count);
        }
    }

    // No context without return
    impl<Func, $($param,)*> CommandFactory<($($param,)*), ()> for Func
    where
        Func: Fn($($param),*),
        $($param: FromArma,)*
    {
        #[allow(non_snake_case)]
        unsafe fn call(&self, _: Context, _output: *mut libc::c_char, _size: libc::c_int, args: Option<*mut *mut i8>, count: Option<libc::c_int>) -> libc::c_int{
            let count = count.unwrap_or_else(|| 0);
            if count != $c {
                println!("Invalid number of arguments: expected {}, got {}", $c, count);
                return format!("2{}", count).parse::<libc::c_int>().unwrap();
            }
            if $c != 0 {
                #[allow(unused_variables, unused_mut)]
                let mut argv: Vec<String> = {
                    let argv: &[*mut libc::c_char; $c] = &*(args.unwrap() as *const [*mut i8; $c]);
                    let mut argv = argv
                    .to_vec()
                    .into_iter()
                    .map(|s|
                        std::ffi::CStr::from_ptr(s)
                        .to_string_lossy()
                        .trim_matches('\"')
                        .to_owned()
                    )
                    .collect::<Vec<String>>();
                    argv.reverse();
                    argv
                };
                #[allow(unused_variables, unused_mut)] // Caused by the 0 loop
                let mut c = 0;
                #[allow(unused_assignments, clippy::eval_order_dependence)]
                (self)($(
                    if let Ok(val) = $param::from_arma(argv.pop().unwrap()) {
                        c += 1;
                        val
                    } else {
                        return format!("3{}", c).parse::<libc::c_int>().unwrap()
                    },
                )*);
                0
            } else {
                (self)($($param::from_arma("".to_string()).unwrap(),)*);
                0
            }
        }
    }

    // Context without return
    impl<Func, $($param,)*> CommandFactory<(Context, $($param,)*), ()> for Func
    where
        Func: Fn(Context, $($param),*),
        $($param: FromArma,)*
    {
        #[allow(non_snake_case)]
        unsafe fn call(&self, context: Context, _output: *mut libc::c_char, _size: libc::c_int, args: Option<*mut *mut i8>, count: Option<libc::c_int>) -> libc::c_int{
            let count = count.unwrap_or_else(|| 0);
            if count != $c {
                println!("Invalid number of arguments: expected {}, got {}", $c, count);
                return format!("2{}", count).parse::<libc::c_int>().unwrap();
            }
            if $c != 0 {
                #[allow(unused_variables, unused_mut)]
                let mut argv: Vec<String> = {
                    let argv: &[*mut libc::c_char; $c] = &*(args.unwrap() as *const [*mut i8; $c]);
                    let mut argv = argv
                    .to_vec()
                    .into_iter()
                    .map(|s|
                        std::ffi::CStr::from_ptr(s)
                        .to_string_lossy()
                        .trim_matches('\"')
                        .to_owned()
                    )
                    .collect::<Vec<String>>();
                    argv.reverse();
                    argv
                };
                #[allow(unused_variables, unused_mut)] // Caused by the 0 loop
                let mut c = 0;
                #[allow(unused_assignments, clippy::eval_order_dependence)]
                (self)(context,
                $(
                    if let Ok(val) = $param::from_arma(argv.pop().unwrap()) {
                        c += 1;
                        val
                    } else {
                        return format!("3{}", c).parse::<libc::c_int>().unwrap()
                    },
                )*);
                0
            } else {
                (self)(context, $($param::from_arma("".to_string()).unwrap(),)*);
                0
            }
        }
    }

    // No context with input and return
    impl<Func, $($param,)* R> CommandFactory<($($param,)*), R> for Func
    where
        R: IntoArma + 'static,
        Func: Fn($($param),*) -> R,
        $($param: FromArma,)*
    {
        #[allow(non_snake_case)]
        unsafe fn call(&self, _: Context, output: *mut libc::c_char, size: libc::c_int, args: Option<*mut *mut i8>, count: Option<libc::c_int>) -> libc::c_int {
            let count = count.unwrap_or_else(|| 0);
            if count != $c {
                println!("Invalid number of arguments: expected {}, got {}", $c, count);
                return format!("2{}", count).parse::<libc::c_int>().unwrap();
            }
            if $c != 0 {
                #[allow(unused_variables, unused_mut)]
                let mut argv: Vec<String> = {
                    let argv: &[*mut libc::c_char; $c] = &*(args.unwrap() as *const [*mut i8; $c]);
                    let mut argv = argv
                    .to_vec()
                    .into_iter()
                    .map(|s|
                        std::ffi::CStr::from_ptr(s)
                        .to_string_lossy()
                        .trim_matches('\"')
                        .to_owned()
                    )
                    .collect::<Vec<String>>();
                    argv.reverse();
                    argv
                };
                crate::write_cstr(
                    {
                        #[allow(unused_variables, unused_mut)] // Caused by the 0 loop
                        let mut c = 0;
                        #[allow(unused_assignments, clippy::eval_order_dependence)]
                        let ret = (self)($(
                            if let Ok(val) = $param::from_arma(argv.pop().unwrap()) {
                                c += 1;
                                val
                            } else {
                                return format!("3{}", c).parse::<libc::c_int>().unwrap()
                            },
                        )*);
                        if let ArmaValue::String(s) = ret.to_arma() {
                            s
                        } else {
                            ret.to_arma().to_string()
                        }
                    },
                    output,
                    size
                );
                0
            } else {
                crate::write_cstr(
                    {
                        let ret = (self)($($param::from_arma("".to_string()).unwrap(),)*);
                        if let ArmaValue::String(s) = ret.to_arma() {
                            s
                        } else {
                            ret.to_arma().to_string()
                        }
                    },
                    output,
                    size
                );
                0
            }
        }
    }

    // Context with input and return
    impl<Func, $($param,)* R> CommandFactory<(Context, $($param,)*), R> for Func
    where
        R: IntoArma + 'static,
        Func: Fn(Context, $($param),*) -> R,
        $($param: FromArma,)*
    {
        #[allow(non_snake_case)]
        unsafe fn call(&self, context: Context, output: *mut libc::c_char, size: libc::c_int, args: Option<*mut *mut i8>, count: Option<libc::c_int>) -> libc::c_int {
            let count = count.unwrap_or_else(|| 0);
            if count != $c {
                println!("Invalid number of arguments: expected {}, got {}", $c, count);
                return format!("2{}", count).parse::<libc::c_int>().unwrap();
            }
            if $c != 0 {
                #[allow(unused_variables, unused_mut)]
                let mut argv: Vec<String> = {
                    let argv: &[*mut libc::c_char; $c] = &*(args.unwrap() as *const [*mut i8; $c]);
                    let mut argv = argv
                    .to_vec()
                    .into_iter()
                    .map(|s|
                        std::ffi::CStr::from_ptr(s)
                        .to_string_lossy()
                        .trim_matches('\"')
                        .to_owned()
                    )
                    .collect::<Vec<String>>();
                    argv.reverse();
                    argv
                };
                crate::write_cstr(
                    {
                        #[allow(unused_variables, unused_mut)] // Caused by the 0 loop
                        let mut c = 0;
                        #[allow(unused_assignments, clippy::eval_order_dependence)]
                        let ret = (self)(context, $(
                            if let Ok(val) = $param::from_arma(argv.pop().unwrap()) {
                                c += 1;
                                val
                            } else {
                                return format!("3{}", c).parse::<libc::c_int>().unwrap()
                            },
                        )*);
                        if let ArmaValue::String(s) = ret.to_arma() {
                            s
                        } else {
                            ret.to_arma().to_string()
                        }
                    },
                    output,
                    size
                );
                0
            } else {
                crate::write_cstr(
                    {
                        let ret = (self)(context, $($param::from_arma("".to_string()).unwrap(),)*);
                        if let ArmaValue::String(s) = ret.to_arma() {
                            s
                        } else {
                            ret.to_arma().to_string()
                        }
                    },
                    output,
                    size
                );
                0
            }
        }
    }
});

factory_tuple! { 0, }
factory_tuple! { 1, A }
factory_tuple! { 2, A B }
factory_tuple! { 3, A B C }
factory_tuple! { 4, A B C D }
factory_tuple! { 5, A B C D E }
factory_tuple! { 6, A B C D E F }
factory_tuple! { 7, A B C D E F G }
factory_tuple! { 8, A B C D E F G H }
factory_tuple! { 9, A B C D E F G H I }
factory_tuple! { 10, A B C D E F G H I J }
factory_tuple! { 11, A B C D E F G H I J K }
factory_tuple! { 12, A B C D E F G H I J K L }
