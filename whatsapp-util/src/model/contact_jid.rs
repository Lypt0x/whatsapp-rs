#![allow(unused)]
#![allow(non_snake_case)]

use anyhow::{bail, Result};

macro_rules! declare_server {
    (
        $($name:ident => $val:expr)*
    ) => {
        #[derive(serde::Serialize, Copy, Clone)]
        pub enum Server {
            $(
                $name,
            )*
        }

        impl Server {
            pub fn address(&self) -> &'static str {
                match self {
                    $(
                        Self::$name => $val,
                    )*
                }
            }

            pub fn of(input: &str) -> Option<Self> {
                for (this, identifier) in Self::addresses() {
                    if input.ends_with(identifier) {
                        return Some(*this)
                    }
                }

                None
            }

            pub fn addresses() -> &'static [(Self, &'static str)] {
                &[
                    $((Self::$name, $val),)*
                ]
            }
        }
    };
}

#[derive(serde::Serialize)]
pub struct ContactJid {
    pub user: String,
    pub server: Server,
    pub device: u32,
    pub agent: u32,
}

impl ContactJid {
    pub fn from_companion(jid: String, device: u32, agent: u32) -> Self {
        Self {
            user: Self::without_server(jid),
            server: Server::Whatsapp,
            device,
            agent,
        }
    }

    pub fn from_complex(jid: String, server: Server) -> Result<Self> {
        let complex_user = Self::without_server(jid);
        if complex_user.is_empty() {
            return Ok(Self {
                user: String::new(),
                server,
                device: 0,
                agent: 0,
            });
        }

        if complex_user.contains(':') {
            if let Some((user, device)) = complex_user.split_once(':') {
                if user.contains('_') {
                    if let Some((user, agent)) = user.split_once('_') {
                        return Ok(Self {
                            user: user.to_owned(),
                            server,
                            device: device.parse()?,
                            agent: agent.parse()?,
                        });
                    }

                    return Ok(Self {
                        user: user.to_owned(),
                        server,
                        device: device.parse()?,
                        agent: 0,
                    });
                }
            }
        }

        if !complex_user.contains('_') {
            return Ok(Self {
                user: complex_user.to_owned(),
                server,
                device: 0,
                agent: 0,
            });
        }

        if let Some((user, agent)) = complex_user.split_once('_') {
            return Ok(Self {
                user: user.to_owned(),
                server,
                device: 0,
                agent: agent.parse()?,
            });
        }

        bail!("Could not parse jid")
    }

    pub fn is_companion(&self) -> bool {
        self.device != 0
    }

    fn without_server(mut jid: String) -> String {
        if jid.is_empty() {
            return String::new();
        }

        for (_, address) in Server::addresses() {
            jid = jid.replace(&format!("@{address}"), "");
        }

        jid
    }
}

declare_server! {
    User => "c.us"
    Group => "g.us"
    Broadcast => "broadcast"
    Call => "call"
    Whatsapp => "s.whatsapp.net"

    Companion => ""
    Business => ""
    Server => ""
    Announcement => ""
    Status => ""
    Unknown => ""
}
