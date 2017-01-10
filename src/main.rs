extern crate libc;

mod cec;

fn main() {
    let mut conn = cec::Connection::new().unwrap();
    conn.init().unwrap();
}
