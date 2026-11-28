//! Per-account logins, ordered by identity for lookup and v1 checkpoints.

use super::{Identity, Login};

pub(super) const MAX_LOGINS: u8 = 8;

#[derive(Default)]
pub(super) struct LoginRows(Vec<(Identity, Login)>);

impl LoginRows {
    fn position(&self, key: &Identity) -> Result<usize, usize> {
        self.0.binary_search_by(|(identity, _)| identity.cmp(key))
    }

    pub(super) fn get(&self, key: &Identity) -> Option<&Login> {
        self.position(key).ok().map(|index| &self.0[index].1)
    }

    pub(super) fn get_mut(&mut self, key: &Identity) -> Option<&mut Login> {
        self.position(key).ok().map(|index| &mut self.0[index].1)
    }

    pub(super) fn insert(&mut self, key: Identity, login: Login) {
        match self.position(&key) {
            Ok(index) => self.0[index].1 = login,
            Err(index) => self.0.insert(index, (key, login)),
        }
    }

    pub(super) fn remove(&mut self, key: &Identity) -> Option<Login> {
        self.position(key).ok().map(|index| self.0.remove(index).1)
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &Login> {
        self.0.iter().map(|(_, login)| login)
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&Identity, &Login)> {
        self.0.iter().map(|(key, login)| (key, login))
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::ops::Index<&Identity> for LoginRows {
    type Output = Login;
    fn index(&self, key: &Identity) -> &Login {
        self.get(key).expect("indexed login")
    }
}

impl serde::Serialize for LoginRows {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.iter())
    }
}

impl<'de> serde::Deserialize<'de> for LoginRows {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct LoginVisitor;
        impl<'de> serde::de::Visitor<'de> for LoginVisitor {
            type Value = LoginRows;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a map of at most eight distinct login identities")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<LoginRows, A::Error> {
                let mut rows = LoginRows::default();
                while let Some(key) = map.next_key()? {
                    if rows.len() == usize::from(MAX_LOGINS) {
                        return Err(serde::de::Error::custom("too many account logins"));
                    }
                    let Err(index) = rows.position(&key) else {
                        return Err(serde::de::Error::custom("duplicate login identity"));
                    };
                    rows.0.insert(index, (key, map.next_value()?));
                }
                Ok(rows)
            }
        }
        decoder.deserialize_map(LoginVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stable_upgrade::ISSUER_LIMITS;
    use ic_wire::Encoding;

    struct LoginMap<'a>(&'a [Identity], &'a Login);

    impl serde::Serialize for LoginMap<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_map(self.0.iter().map(|key| (key, self.1)))
        }
    }

    #[test]
    fn checkpoint_admits_eight_distinct_keys_and_refuses_duplicates_or_overflow() {
        let issuer = super::super::tests::completed_login();
        let login = issuer
            .accounts
            .values()
            .next()
            .unwrap()
            .logins
            .values()
            .next()
            .unwrap();
        let decode = |keys: &[Identity]| {
            let bytes =
                ic_wire::encode(&LoginMap(keys, login), Encoding::Payload, ISSUER_LIMITS).unwrap();
            ic_wire::decode::<LoginRows>(&bytes, Encoding::Payload, ISSUER_LIMITS)
        };
        let keys: Vec<_> = (0..9).rev().map(|epoch| (epoch, [0; 32])).collect();
        let rows = decode(&keys[..8]).unwrap();
        assert!(rows.iter().map(|(key, _)| key).is_sorted());
        assert_eq!(rows.len(), 8);
        assert!(decode(&keys).is_err());
        assert!(decode(&[keys[0], keys[0]]).is_err());
    }
}
