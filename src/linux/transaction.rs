pub(crate) trait Store {
    fn read(&self, key: &str) -> Result<String, String>;
    fn write(&self, key: &str, value: &str) -> Result<(), String>;
}

pub(crate) fn apply<S: Store>(store: &S, changes: &[(&str, String)]) -> Result<(), String> {
    let previous = changes
        .iter()
        .map(|(key, _)| store.read(key))
        .collect::<Result<Vec<_>, _>>()?;

    for (index, (key, value)) in changes.iter().enumerate() {
        if let Err(error) = store.write(key, value) {
            for ((key, _), old) in changes[..index].iter().zip(&previous).rev() {
                let _ = store.write(key, old);
            }
            return Err(error);
        }
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
        fn read(&self, key: &str) -> Result<String, String> {
            self.get(key).ok_or_else(|| format!("No such key {}", key))
        }

        fn write(&self, key: &str, value: &str) -> Result<(), String> {
            self.writes.borrow_mut().push((key.to_string(), value.to_string()));
            if self.rejected_key == Some(key) {
                return Err(format!("Unable to set {}", key));
            }
            self.values.borrow_mut().insert(key.to_string(), value.to_string());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::apply;
    use super::fake::FakeStore;

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
    fn failed_read_writes_nothing() {
        let store = FakeStore::with(&[("a", "1")]);
        assert!(apply(&store, &[("a", "10".into()), ("missing", "x".into())]).is_err());
        assert!(store.writes.borrow().is_empty());
    }
}
