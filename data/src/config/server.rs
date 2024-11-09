use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use irc::connection;
use serde::{Deserialize, Deserializer};

use crate::config;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Server {
    /// The client's nickname.
    pub nickname: String,
    /// The client's NICKSERV password.
    pub nick_password: Option<String>,
    /// The client's NICKSERV password file.
    pub nick_password_file: Option<String>,
    /// The client's NICKSERV password command.
    pub nick_password_command: Option<String>,
    /// The server's NICKSERV IDENTIFY syntax.
    pub nick_identify_syntax: Option<IdentifySyntax>,
    /// Alternative nicknames for the client, if the default is taken.
    #[serde(default)]
    pub alt_nicks: Vec<String>,
    /// The client's username.
    pub username: Option<String>,
    /// The client's real name.
    pub realname: Option<String>,
    /// The server to connect to.
    pub server: String,
    /// The port to connect on.
    #[serde(default = "default_tls_port")]
    pub port: u16,
    /// The password to connect to the server.
    pub password: Option<String>,
    /// The file with the password to connect to the server.
    pub password_file: Option<String>,
    /// The command which outputs a password to connect to the server.
    pub password_command: Option<String>,
    /// A list of channels to join on connection.
    #[serde(default)]
    pub channels: Vec<String>,
    /// A mapping of channel names to keys for join-on-connect.
    #[serde(default)]
    pub channel_keys: HashMap<String, String>,
    /// The amount of inactivity in seconds before the client will ping the server.
    #[serde(default = "default_ping_time")]
    pub ping_time: u64,
    /// The amount of time in seconds for a client to reconnect due to no ping response.
    #[serde(default = "default_ping_timeout")]
    pub ping_timeout: u64,
    /// The amount of time in seconds before attempting to reconnect to the server when disconnected.
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay: u64,
    /// Whether the client should use NickServ GHOST to reclaim its primary nickname if it is in
    /// use. This has no effect if `nick_password` is not set.
    #[serde(default)]
    pub should_ghost: bool,
    /// The command(s) that should be sent to NickServ to recover a nickname. The nickname and
    /// password will be appended in that order after the command.
    /// E.g. `["RECOVER", "RELEASE"]` means `RECOVER nick pass` and `RELEASE nick pass` will be sent
    /// in that order.
    #[serde(default = "default_ghost_sequence")]
    pub ghost_sequence: Vec<String>,
    /// User modestring to set on connect. Example: "+RB-x"
    pub umodes: Option<String>,
    /// Whether or not to use TLS.
    /// Clients will automatically panic if this is enabled without TLS support.
    #[serde(default = "default_use_tls")]
    pub use_tls: bool,
    /// On `true`, all certificate validations are skipped. Defaults to `false`.
    #[serde(default)]
    pub dangerously_accept_invalid_certs: bool,
    /// The path to the root TLS certificate for this server in PEM format.
    root_cert_path: Option<PathBuf>,
    /// Sasl authentication
    pub sasl: Option<Sasl>,
    /// Commands which are executed once connected.
    #[serde(default)]
    pub on_connect: Vec<String>,
    /// Enable WHO polling. Defaults to `true`.
    #[serde(default = "default_who_poll_enabled")]
    pub who_poll_enabled: bool,
    /// WHO poll interval for servers without away-notify.
    #[serde(
        default = "default_who_poll_interval",
        deserialize_with = "deserialize_duration_from_u64"
    )]
    pub who_poll_interval: Duration,
    /// WHO retry interval for servers without away-notify.
    #[serde(
        default = "default_who_retry_interval",
        deserialize_with = "deserialize_duration_from_u64"
    )]
    pub who_retry_interval: Duration,
    /// A list of nicknames to monitor (if MONITOR is supported by the server).
    #[serde(default)]
    pub monitor: Vec<String>,
    #[serde(default = "default_chathistory")]
    pub chathistory: bool,
}

impl Server {
    pub fn new(
        server: String,
        port: Option<u16>,
        nickname: String,
        channels: Vec<String>,
        use_tls: bool,
    ) -> Self {
        Self {
            nickname,
            server,
            port: port.unwrap_or(if use_tls {
                default_tls_port()
            } else {
                default_port()
            }),
            channels,
            use_tls,
            dangerously_accept_invalid_certs: false,
            ..Default::default()
        }
    }

    pub fn connection(&self, proxy: Option<config::Proxy>) -> connection::Config {
        let security = if self.use_tls {
            connection::Security::Secured {
                accept_invalid_certs: self.dangerously_accept_invalid_certs,
                root_cert_path: self.root_cert_path.as_ref(),
                client_cert_path: self.sasl.as_ref().and_then(Sasl::external_cert),
                client_key_path: self.sasl.as_ref().and_then(Sasl::external_key),
            }
        } else {
            connection::Security::Unsecured
        };

        connection::Config {
            server: &self.server,
            port: self.port,
            security,
            proxy: proxy.map(From::from),
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            nickname: Default::default(),
            nick_password: Default::default(),
            nick_password_file: Default::default(),
            nick_password_command: Default::default(),
            nick_identify_syntax: Default::default(),
            alt_nicks: Default::default(),
            username: Default::default(),
            realname: Default::default(),
            server: Default::default(),
            port: default_tls_port(),
            password: Default::default(),
            password_file: Default::default(),
            password_command: Default::default(),
            channels: Default::default(),
            channel_keys: Default::default(),
            ping_time: default_ping_time(),
            ping_timeout: default_ping_timeout(),
            reconnect_delay: default_reconnect_delay(),
            should_ghost: Default::default(),
            ghost_sequence: default_ghost_sequence(),
            umodes: Default::default(),
            use_tls: default_use_tls(),
            dangerously_accept_invalid_certs: Default::default(),
            root_cert_path: Default::default(),
            sasl: Default::default(),
            on_connect: Default::default(),
            who_poll_enabled: default_who_poll_enabled(),
            who_poll_interval: default_who_poll_interval(),
            who_retry_interval: default_who_retry_interval(),
            monitor: Default::default(),
            chathistory: default_chathistory(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdentifySyntax {
    NickPassword,
    PasswordNick,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sasl {
    Plain {
        /// Account name
        username: String,
        /// Account password,
        password: Option<String>,
        /// Account password file
        password_file: Option<String>,
        /// Account password command
        password_command: Option<String>,
    },
    External {
        /// The path to PEM encoded X509 user certificate for external auth
        cert: PathBuf,
        /// The path to PEM encoded PKCS#8 private key corresponding to the user certificate for external auth
        key: Option<PathBuf>,
    },
}

impl Sasl {
    pub fn command(&self) -> &'static str {
        match self {
            Sasl::Plain { .. } => "PLAIN",
            Sasl::External { .. } => "EXTERNAL",
        }
    }

    pub fn param(&self) -> String {
        match self {
            Sasl::Plain {
                username, password, ..
            } => {
                use base64::engine::Engine;

                let password = password
                    .as_ref()
                    .expect("SASL password must exist at this point!");

                base64::engine::general_purpose::STANDARD
                    .encode(format!("{username}\x00{username}\x00{password}"))
            }
            Sasl::External { .. } => "+".into(),
        }
    }

    fn external_cert(&self) -> Option<&PathBuf> {
        if let Self::External { cert, .. } = self {
            Some(cert)
        } else {
            None
        }
    }

    fn external_key(&self) -> Option<&PathBuf> {
        if let Self::External { key, .. } = self {
            key.as_ref()
        } else {
            None
        }
    }
}

fn deserialize_duration_from_u64<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::from_secs(seconds.clamp(5, 3600)))
}

fn default_use_tls() -> bool {
    true
}

fn default_tls_port() -> u16 {
    6697
}

fn default_port() -> u16 {
    6667
}

fn default_ping_time() -> u64 {
    180
}

fn default_ping_timeout() -> u64 {
    20
}

fn default_reconnect_delay() -> u64 {
    10
}

fn default_ghost_sequence() -> Vec<String> {
    vec!["REGAIN".into()]
}

fn default_who_poll_enabled() -> bool {
    true
}

fn default_who_poll_interval() -> Duration {
    Duration::from_secs(180)
}

fn default_who_retry_interval() -> Duration {
    Duration::from_secs(10)
}

fn default_chathistory() -> bool {
    true
}
