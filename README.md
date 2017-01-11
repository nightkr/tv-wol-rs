# TV-WoL-RS

Wake on LAN for HDMI-CEC. Listens on a random TCP port, turns on the TV when there is at least one connection, turns it off when there are no connections.

This is announced as a Zeroconf service of the type `_tv-wol._tcp`.

Rust rewrite of [TV-WoL](https://github.com/teozkr/tv-wol).
