#![no_std]
fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[no_mangle]
fn main() {
    let x = add(10, 21);
    let _y = add(x, x);
}
