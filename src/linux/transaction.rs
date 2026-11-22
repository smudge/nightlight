pub(crate) trait Store {
    fn read(&self, key: &str) -> Result<Option<String>, String>;
    fn write(&self, key: &str, value: &str) -> Result<(), String>;

    fn remove(&self, _key: &str) -> Result<(), String> {
        Ok(())
    }

    fn require(&self, key: &str) -> Result<String, String> {
        self.read(key)?.ok_or_else(|| format!("{} is not set", key))
    }
}

pub(crate) fn apply<S: Store>(store: &S, changes: &[(&str, String)]) -> Result<(), String> {
    let previous = changes
        .iter()
        .map(|(key, _)| store.read(key))
        .collect::<Result<Vec<_>, _>>()?;

    let mut written: Vec<(&str, &Option<String>)> = vec![];
    for ((key, value), old) in changes.iter().zip(&previous) {
        if old.as_deref() == Some(value.as_str()) {
            continue;
        }
        if let Err(error) = store.write(key, value) {
            for (key, old) in written.into_iter().rev() {
                let _ = match old {
                    Some(old) => store.write(key, old),
                    None => store.remove(key),
                };
            }
            return Err(error);
        }
        written.push((key, old));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod fake {
    use super::Store;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    pub(crate) struct FakeStore {
        pub values: RefCell<HashMap<String, String>>,
        pub rejected_key: Option<&'static str>,
        pub writes: RefCell<Vec<(String, String)>>,
    }

    impl FakeStore {
        pub fn with(values: &[(&str, &str)]) -> FakeStore {
            FakeStore {
                values: RefCell::new(
                    values.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                ),
                ..Default::default()
            }
        }

        pub fn get(&self, key: &str) -> Option<String> {
            self.values.borrow().get(key).cloned()
        }
    }

    impl Store for FakeStore {
        fn read(&self, key: &str) -> Result<Option<String>, String> {
            Ok(self.get(key))
        }

        fn write(&self, key: &str, value: &str) -> Result<(), String> {
            self.writes.borrow_mut().push((key.to_string(), value.to_string()));
            if self.rejected_key == Some(key) {
                return Err(format!("Unable to set {}", key));
            }
            self.values.borrow_mut().insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn remove(&self, key: &str) -> Result<(), String> {
            self.values.borrow_mut().remove(key);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::FakeStore;
    use super::{apply, Store};

    #[test]
    fn applies_every_change_in_order() {
        let store = FakeStore::with(&[("a", "1"), ("b", "2")]);
        apply(&store, &[("a", "10".into()), ("b", "20".into())]).unwrap();
        assert_eq!(store.get("a").unwrap(), "10");
        assert_eq!(store.get("b").unwrap(), "20");
    }

    #[test]
    fn failed_write_restores_earlier_keys() {
        let store = FakeStore {
            rejected_key: Some("c"),
            ..FakeStore::with(&[("a", "1"), ("b", "2"), ("c", "3")])
        };
        let changes = [("a", "10".into()), ("b", "20".into()), ("c", "30".into())];
        assert_eq!(apply(&store, &changes), Err("Unable to set c".into()));
        assert_eq!(store.get("a").unwrap(), "1");
        assert_eq!(store.get("b").unwrap(), "2");
        assert_eq!(store.get("c").unwrap(), "3");
        assert_eq!(store.writes.borrow().last().unwrap(), &("a".into(), "1".into()));
    }

    #[test]
    fn failed_write_unsets_keys_that_were_unset() {
        let store = FakeStore {
            rejected_key: Some("b"),
            ..FakeStore::with(&[("b", "2")])
        };
        assert!(apply(&store, &[("a", "10".into()), ("b", "20".into())]).is_err());
        assert_eq!(store.get("a"), None);
    }

    #[test]
    fn unchanged_keys_are_not_written() {
        let store = FakeStore {
            rejected_key: Some("c"),
            ..FakeStore::with(&[("a", "1"), ("b", "2"), ("c", "3")])
        };
        apply(&store, &[("a", "1".into()), ("b", "20".into())]).unwrap();
        assert_eq!(*store.writes.borrow(), vec![("b".to_string(), "20".to_string())]);

        store.writes.borrow_mut().clear();
        assert!(apply(&store, &[("a", "10".into()), ("b", "20".into()), ("c", "30".into())]).is_err());
        assert_eq!(
            *store.writes.borrow(),
            vec![
                ("a".to_string(), "10".to_string()),
                ("c".to_string(), "30".to_string()),
                ("a".to_string(), "1".to_string()),
            ]
        );
    }

    #[test]
    fn required_keys_must_be_set() {
        let store = FakeStore::with(&[("a", "1")]);
        assert_eq!(store.require("a"), Ok("1".into()));
        assert_eq!(store.require("b"), Err("b is not set".into()));
    }
}
