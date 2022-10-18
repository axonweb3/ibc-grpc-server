use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::{from_utf8, FromStr};

use ibc::core::ics24_host::{path, validate::validate_identifier, Path as IbcPath};

use crate::{Result, ServerError};

macro_rules! impl_into_path_for {
    ($($path:ty),+) => {
        $(impl From<$path> for Path {
            fn from(ibc_path: $path) -> Self {
                Self::try_from(ibc_path.to_string()).unwrap() // safety - `IbcPath`s are correct-by-construction
            }
        })+
    };
}

impl_into_path_for!(
    path::ClientTypePath,
    path::ClientStatePath,
    path::ClientConsensusStatePath,
    path::ConnectionsPath,
    path::ClientConnectionsPath,
    path::ChannelEndsPath,
    path::SeqSendsPath,
    path::SeqRecvsPath,
    path::SeqAcksPath,
    path::CommitmentsPath,
    path::ReceiptsPath,
    path::AcksPath
);

/// A new type representing a valid ICS024 identifier.
/// Implements `Deref<Target=String>`.
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Identifier(String);

impl Identifier {
    /// Identifiers MUST be non-empty (of positive integer length).
    /// Identifiers MUST consist of characters in one of the following
    /// categories only:
    /// * Alphanumeric
    /// * `.`, `_`, `+`, `-`, `#`
    /// * `[`, `]`, `<`, `>`
    fn validate(s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();

        // give a `min` parameter of 0 here to allow id's of arbitrary
        // length as inputs; `validate_identifier` itself checks for
        // empty inputs and returns an error as appropriate
        validate_identifier(s, 0, s.len()).map_err(ServerError::ValidateIdentifier)
    }
}

impl Deref for Identifier {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for Identifier {
    type Error = ServerError;

    fn try_from(s: String) -> Result<Self> {
        Identifier::validate(&s).map(|_| Self(s))
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StoreHeight {
    Latest,
    Stable(u64),
}

/// A new type representing a valid ICS024 `Path`.
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Path(Vec<Identifier>);

impl Path {
    pub fn starts_with(&self, prefix: &Path) -> bool {
        KeyPrefix::from(self)
            .as_ref()
            .starts_with(KeyPrefix::from(prefix).as_ref())
    }
}

impl TryFrom<String> for Path {
    type Error = ServerError;

    fn try_from(s: String) -> Result<Self> {
        let mut identifiers = vec![];
        let parts = s.split('/'); // split will never return an empty iterator
        for part in parts {
            identifiers.push(Identifier::try_from(part.to_owned())?);
        }
        Ok(Self(identifiers))
    }
}

impl TryFrom<&[u8]> for Path {
    type Error = ServerError;

    fn try_from(value: &[u8]) -> Result<Self> {
        let s = from_utf8(value).map_err(ServerError::FromUtf8)?;
        s.to_owned().try_into()
    }
}

impl From<Identifier> for Path {
    fn from(id: Identifier) -> Self {
        Self(vec![id])
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|iden| iden.as_str().to_owned())
                .collect::<Vec<String>>()
                .join("/")
        )
    }
}

impl TryFrom<Path> for IbcPath {
    type Error = path::PathError;

    fn try_from(path: Path) -> std::result::Result<Self, Self::Error> {
        Self::from_str(path.to_string().as_str())
    }
}

impl From<IbcPath> for Path {
    fn from(ibc_path: IbcPath) -> Self {
        Self::try_from(ibc_path.to_string()).unwrap() // safety - `IbcPath`s are correct-by-construction
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct KeyPrefix(String);

impl From<&Path> for KeyPrefix {
    fn from(ibc_path: &Path) -> Self {
        let strs: Vec<String> = ibc_path.0.iter().map(|x| x.to_string()).collect();
        KeyPrefix(strs.join("/"))
    }
}

impl From<&KeyPrefix> for String {
    fn from(key_prefix: &KeyPrefix) -> Self {
        key_prefix.0.clone()
    }
}

impl AsRef<[u8]> for KeyPrefix {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::ics02_client::client_type::ClientType;
    use ibc::core::ics24_host::{identifier::ClientId, path::ClientTypePath};

    use super::*;

    #[test]
    fn starts_with_test_case01() {
        let dst_path: Path = "clients/07-tendermint-1033".to_owned().try_into().unwrap();
        let client_id = ClientId::new(ClientType::Tendermint, 103344).unwrap();
        let path = ClientTypePath(client_id);
        let src_path: Path = Path::from(path);

        assert!(src_path.starts_with(&dst_path));
    }

    #[test]

    fn starts_with_test_case02() {
        use ibc::core::ics24_host::{identifier::ClientId, path::ClientStatePath};
        let dst_path = "clients/07-tend".to_owned().try_into().unwrap();
        let client_id = ClientId::new(ClientType::Tendermint, 1033).unwrap();
        let path = ClientStatePath(client_id);
        let src_path: Path = Path::from(path);

        assert!(src_path.starts_with(&dst_path));
    }

    #[test]

    fn starts_with_test_case03() {
        use ibc::core::ics24_host::{identifier::ClientId, path::ClientStatePath};
        let dst_path = "clients".to_owned().try_into().unwrap();
        let client_id = ClientId::new(ClientType::Tendermint, 1033).unwrap();
        let path = ClientStatePath(client_id);
        let src_path: Path = Path::from(path);

        assert!(src_path.starts_with(&dst_path));
    }

    #[test]

    fn test_key_prefix_from_path_case01() {
        let src = String::from("connections");
        let src_path: Path = src
            .clone()
            .try_into()
            .expect("'connections' expected to be a valid Path");
        let prefix_key = KeyPrefix::from(&src_path);
        assert_eq!(src, String::from(&prefix_key))
    }

    #[test]

    fn test_key_prefix_from_path_case02() {
        let src = String::from("channelEnds/ports");
        let src_path: Path = src
            .clone()
            .try_into()
            .expect("'channelEnds/ports' expected to be a valid Path");
        let prefix_key = KeyPrefix::from(&src_path);
        assert_eq!(src, String::from(&prefix_key))
    }

    #[test]

    fn test_key_prefix_from_path_case03() {
        let src = format!("clients/{}/consensusStates", "123client_id");
        let src_path = src
            .clone()
            .try_into()
            .expect("'channelEnds/ports' expected to be a valid Path");
        let prefix_key = KeyPrefix::from(&src_path);
        assert_eq!(src, String::from(&prefix_key))
    }
}
