use super::errors::Error;
use klave;

pub trait DB {
    fn get<K>(&self, key: K) -> Result<Option<Vec<u8>>, Error>
    where
        K: Into<String>;

    fn put<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: Into<String>,
        V: AsRef<[u8]>;
}

#[derive(Debug)]
pub struct FileDB {
    store_table: String,
}

impl FileDB {
    pub fn open(store_table: String) -> Result<Self, Error> {
        Ok(Self { 
            store_table
        })
    }
}

impl DB for FileDB {
    fn get<K>(&self, key: K) -> Result<Option<Vec<u8>>, Error>
    where
        K: Into<String>,
    {
        match klave::ledger::get_table(self.store_table.as_str()).get(key.into().as_str()) {
            Ok(v) => Ok(Some(v.into())),
            Err(e) => return Err(Error::Other { description: e.to_string() })
        }
    }

    fn put<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: Into<String>,
        V: AsRef<[u8]>,
    {
        let key = key.into();
        let value = value.as_ref().to_vec();

        match klave::ledger::get_table(self.store_table.as_str()).set(key.as_str(), &value) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Other { description: e.to_string() }),
        }
    }
}
