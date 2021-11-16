use super::error::Error;
use rocksdb::{IteratorMode, MergeOperands, Options, DB};
use std::convert::TryInto;
use std::path::Path;

pub struct Mapping {
    db: DB,
}

impl Mapping {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Mapping, Error> {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.set_merge_operator_associative("merge", Self::merge);
        let db = DB::open(&options, path)?;

        Ok(Mapping { db })
    }

    pub fn get_estimated_key_count(&self) -> Result<u64, Error> {
        Ok(self
            .db
            .property_int_value("rocksdb.estimate-num-keys")?
            .unwrap())
    }

    pub fn get_key_counts(&self) -> (u64, u64) {
        let mut id_keys = 0;
        let mut screen_name_keys = 0;

        let iter = self.db.iterator(IteratorMode::Start);

        for (key, _) in iter {
            if key[0] == 0 {
                id_keys += 1;
            } else if key[0] == 1 {
                screen_name_keys += 1;
            } else {
                panic!("Invalid key prefix: {}", key[0]);
            }
        }

        (id_keys, screen_name_keys)
    }

    pub fn lookup_by_id(&self, id: u64) -> Result<Vec<String>, Error> {
        let value = self.db.get_pinned(Self::id_to_key(id))?;

        if let Some(value) = value {
            let mut result = Vec::with_capacity(1);
            let mut i = 0;

            while i < value.len() {
                let len = value[i] as usize;
                let next = std::str::from_utf8(&value[i + 1..i + 1 + len]).unwrap();

                result.push(next.to_string());
                i += len + 1;
            }

            Ok(result)
        } else {
            Ok(vec![])
        }
    }

    pub fn lookup_by_screen_name(&self, screen_name: &str) -> Result<Vec<u64>, Error> {
        let form = screen_name.to_lowercase();
        let value = self.db.get_pinned(Self::screen_name_to_key(&form))?;

        if let Some(value) = value {
            let mut result = Vec::with_capacity(1);
            let mut i = 0;

            while i < value.len() {
                let next = u64::from_be_bytes(value[i..i + 8].try_into().unwrap());

                result.push(next);
                i += 8;
            }

            Ok(result)
        } else {
            Ok(vec![])
        }
    }

    pub fn insert_pair(&self, id: u64, screen_name: &str) -> Result<(), Error> {
        let as_bytes = screen_name.as_bytes();
        let mut value = Vec::with_capacity(as_bytes.len() + 1);
        value.push(as_bytes.len() as u8);
        value.extend_from_slice(as_bytes);

        self.db.merge(Self::id_to_key(id), value)?;
        self.db
            .merge(Self::screen_name_to_key(screen_name), id.to_be_bytes())?;

        Ok(())
    }

    fn id_to_key(id: u64) -> Vec<u8> {
        let mut key = Vec::with_capacity(9);
        key.push(0);
        key.extend_from_slice(&id.to_be_bytes());
        key
    }

    fn screen_name_to_key(screen_name: &str) -> Vec<u8> {
        let form = screen_name.to_lowercase();
        let as_bytes = form.as_bytes();
        let mut key = Vec::with_capacity(as_bytes.len() + 1);
        key.push(1);
        key.extend_from_slice(as_bytes);
        key
    }

    fn merge(
        new_key: &[u8],
        existing_val: Option<&[u8]>,
        operands: &mut MergeOperands,
    ) -> Option<Vec<u8>> {
        let mut new_val = match existing_val {
            Some(bytes) => bytes.to_vec(),
            None => Vec::with_capacity(operands.size_hint().0 * 10 * 8),
        };

        if new_key[0] == 0 {
            for operand in operands {
                Self::merge_n(&mut new_val, operand);
            }
        } else {
            for operand in operands {
                Self::merge_i(&mut new_val, operand);
            }
        }

        Some(new_val)
    }

    fn merge_i(a: &mut Vec<u8>, b: &[u8]) {
        let original_len = a.len();
        let mut i = 0;

        while i < b.len() {
            let next_b = u64::from_be_bytes(b[i..i + 8].try_into().unwrap());

            let mut found = false;
            let mut j = 0;

            while !found && j < original_len {
                let next_a = u64::from_be_bytes(a[j..j + 8].try_into().unwrap());
                found = next_a == next_b;
                j += 8;
            }

            if !found {
                a.extend_from_slice(&b[i..i + 8]);
            }
            i += 8;
        }
    }

    fn merge_n(a: &mut Vec<u8>, b: &[u8]) {
        let original_len = a.len();
        let mut i = 0;

        while i < b.len() {
            let len_b = b[i] as usize;
            let next_b = &b[i + 1..i + 1 + len_b];

            let mut found = false;
            let mut j = 0;

            while !found && j < original_len {
                let len_a = a[j] as usize;
                if len_a == len_b {
                    let next_a = &a[j + 1..j + 1 + len_a];
                    found = next_a == next_b;
                }
                j += len_a + 1;
            }

            if !found {
                a.extend_from_slice(&b[i..i + 1 + len_b]);
            }
            i += len_b + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let dir = tempfile::tempdir().unwrap();
        let db = Mapping::new(dir).unwrap();
        db.insert_pair(123, "foo").unwrap();
        db.insert_pair(123, "bar").unwrap();
        db.insert_pair(456, "foo").unwrap();
        db.insert_pair(123, "foo").unwrap();
        assert_eq!(db.lookup_by_screen_name("foo").unwrap(), vec![123, 456]);
        assert_eq!(db.lookup_by_id(123).unwrap(), vec!["foo", "bar"]);
    }
}
