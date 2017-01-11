use dbus;
use dbus::Message;
use dbus::arg::{Append, IterAppend};
use std::net;

static DBUS_TIMEOUT: i32 = 2000;
static BUS_AVAHI: &'static str = "org.freedesktop.Avahi";
static IF_ENTRY_GROUP: &'static str = "org.freedesktop.Avahi.EntryGroup";
static IF_SERVER: &'static str = "org.freedesktop.Avahi.Server";

fn str_to_err(err: String) -> dbus::Error {
    dbus::Error::new_custom("Failed to create message", err.as_ref())
}

#[allow(dead_code)]
enum AvahiProto {
    INET = 0,
    INET6 = 1,
    UNSPEC = -1
}

impl Append for AvahiProto {
    fn append(self, append: &mut IterAppend) {
        (self as i32).append(append)
    }
}

enum AvahiIf {
    UNSPEC = -1
}

impl Append for AvahiIf {
    fn append(self, append: &mut IterAppend) {
        (self as i32).append(append)
    }
}

pub struct EntryGroup<'a> {
    connection: dbus::Connection,
    path: dbus::Path<'a>
}

impl<'a> EntryGroup<'a> {
    fn new() -> Result<EntryGroup<'a>, dbus::Error> {
        let conn = dbus::Connection::get_private(dbus::BusType::System)?;
        let message = Message::new_method_call(BUS_AVAHI, "/", IF_SERVER, "EntryGroupNew").map_err(str_to_err)?;
        let reply = conn.send_with_reply_and_block(message, DBUS_TIMEOUT)?;
        let path = reply.get1::<dbus::Path>().unwrap();
        Ok(EntryGroup { connection: conn, path: dbus::Path::new(path.as_bytes()).unwrap() })
    }

    fn add_service(&mut self, name: &str, service_type: &str, domain: Option<&str>, host: Option<net::IpAddr>, port: u16) -> Result<(), dbus::Error> {
        let txt: [&[u8]; 0] = [];
        let message = Message::new_method_call(BUS_AVAHI, self.path.clone(), IF_ENTRY_GROUP, "AddService").map_err(str_to_err)?
        .append3(AvahiIf::UNSPEC, AvahiProto::UNSPEC, 0u32)
            .append3(name, service_type, domain.unwrap_or(""))
            .append2(host.map_or("".to_string(), |h| h.to_string()), port)
            .append1(txt.as_ref());
        self.connection.send_with_reply_and_block(message, DBUS_TIMEOUT)?;
        Ok(())
    }

    fn commit(&mut self) -> Result<(), dbus::Error> {
        let message = Message::new_method_call(BUS_AVAHI, self.path.clone(), IF_ENTRY_GROUP, "Commit").map_err(str_to_err)?;
        self.connection.send_with_reply_and_block(message, DBUS_TIMEOUT)?;
        Ok(())
    }
}

impl<'a> Drop for EntryGroup<'a> {
    fn drop(&mut self) {
        let message = Message::new_method_call(BUS_AVAHI, self.path.clone(), IF_ENTRY_GROUP, "Free").map_err(str_to_err).unwrap();
        self.connection.send_with_reply_and_block(message, DBUS_TIMEOUT).unwrap();
    }
}

pub fn register<'a>(addr: net::SocketAddr) -> Result<EntryGroup<'a>, dbus::Error> {
    let mut entry_group = EntryGroup::new()?;
    entry_group.add_service("TV-WoL", "_tv-wol._tcp", None, None, addr.port())?;
    entry_group.commit()?;
    Ok(entry_group)
}
