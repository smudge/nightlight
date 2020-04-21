extern crate objc;

use objc::rc::StrongPtr;
use objc::runtime::{Object, BOOL};
use objc::{class, msg_send, sel, sel_impl};

#[link(name = "CoreBrightness", kind = "framework")]
extern "C" {}

fn main() {
    let client_class = class!(CBBlueLightClient);
    let client = unsafe {
        let obj: *mut Object = msg_send![client_class, alloc];
        let obj: *mut Object = msg_send![obj, init];
        StrongPtr::new(obj)
    };

    let enabled = true as BOOL;
    let result: BOOL = unsafe { msg_send![*client, setEnabled: enabled] };
    if result != enabled {
        eprintln!(
            "WARNING: Attempted to set enabled to {}, but received {}",
            enabled, result
        );
    }
}
