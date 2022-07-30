use crate::protobuf::whatsapp::AppVersion;
use crate::result::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub is_broken: bool,
    pub is_below_soft: bool,

    #[serde(deserialize_with = "super::deserialize_null_default")]
    pub hard_update_time: i64,

    #[serde(deserialize_with = "super::deserialize_null_default")]
    pub beta: String,
    pub current_version: String,
}

impl TryInto<AppVersion> for Version {
    type Error = Error;

    fn try_into(self) -> Result<AppVersion, Self::Error> {
        parse(self).ok_or(Error::IntoError("Malformed version"))
    }
}

pub(crate) fn parse(version: Version) -> Option<AppVersion> {
    let mut arg = version.current_version.splitn(3, '.');

    Some(AppVersion {
        primary: arg.next()?.parse().ok(),
        secondary: arg.next()?.parse().ok(),
        tertiary: arg.next()?.parse().ok(),
        ..Default::default()
    })
}
