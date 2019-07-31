# arma-rs

The easiest way to make extensions for Arma 3.

## Installation

```toml
[dependencies]
arma-rs = { git = "https://github.com/synixebrett/arma-rs", branch = "master" }
```

## Usage

### Hello World

```rust
use arma_rs::{rv, rv_handler};

#[rv]
fn hello() -> &'static str {
    "Hello from Rust!"
}

#[rv_handler]
fn init() {
    println!("Ready to go!");
}

```

Functions can easily be created by using the `rv` attribute. Every extension needs to have a `init()` function with the `rv_handler` attribute. `init()` is called when Arma 3 checks the version number of your extension.  
If you do not require an `init()` just use an empty function.

### Arguments

```rust
#[rv]
fn say_hello(name: String) -> String {
    format!("Hello {}", name)
}
```

`"myExtension" callExtension ["say_hello", ["Rust"]]` => `Hello Rust`

Any type that implements the trait `std::str::FromStr` can be used as an argument.  
Any type that implements the trait `std::str::ToStr` can be used as the return type.

```rust
#[rv]
fn is_arma3(version: u8) -> bool {
    version == 3
}
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## Example

### With arma-rs
```rust 
use arma_rs::{rv, rv_handler};

#[rv]
fn hello() -> &'static str {
    "Hello from Rust!"
}

#[rv]
fn is_arma3(version: u8) -> bool {
    version == 3
}

#[rv_handler]
fn init() {}
```

### Without arma-rs
```rust
extern crate libc;
use libc::c_char;
use libc::strncpy;

use std::ffi::CStr;
use std::ffi::CString;

#[no_mangle]
pub unsafe extern "stdcall" fn RvExtensionVersion(output: *mut c_char, output_size: usize) {
  strncpy(output, CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw(), output_size);
}

#[no_mangle]
pub unsafe extern "stdcall" fn RVExtension(output: *mut c_char, output_size: usize, function: *mut c_char ) {
  let size = output_size - 1;
  let r_function = CStr::from_ptr(function).to_str().unwrap();
  match r_function {
    "hello" => {
      strncpy(output, CString::new("Hello from Rust!").unwrap().into_raw(), size);
    },
    _ => {
      strncpy(output, CString::new("unknown").unwrap().into_raw(), size);
    }
  }
}

#[no_mangle]
pub unsafe extern "stdcall" fn RVExtensionArgs(output: *mut c_char, output_size: usize, function: *mut c_char, args: *mut *mut c_char, arg_count: usize) {
  let size = output_size - 1;
  let r_function = CStr::from_ptr(function).to_str().unwrap();
  match r_function {
    "is_arma3" => {
      if arg_count != 1 {
        strncpy(output, CString::new(format!("Unexpected arg count: {}", arg_count)).unwrap().into_raw(), size);
      } else {
        let argv: &[*mut c_char; 1] = std::mem::transmute(args);
        let version = u8::from_str(CStr::from_ptr(argv[0]).to_str().unwrap().replace("\"","")).unwrap();
        strncpy(output, CString::new((version == 3).to_string()).unwrap().into_raw(), size);
      }
    },
    _ => {
      strncpy(output, CString::new("unknown").unwrap().into_raw(), size);
    }
  }
}
```
