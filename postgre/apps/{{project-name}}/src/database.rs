use klave::{crypto::subtle::{save_key}};
use serde_json::{self, Value};
use serde::{Deserialize, Serialize};

use crate::{crypto::{generate_ecc_crypto_key, encrypt_value}, utils::{flatten_vec_of_vec_values_to_single_string}};

pub(crate) const DATABASE_CLIENT_TABLE: &str = "DatabaseClientTable";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBInputDetails {
    pub host: String,
    pub dbname: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteInput {
    pub database_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseIdInput {
    pub database_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBTable {
    pub database_id: String,
    pub table: String,
    pub columns: Vec<String>,
    pub primary_key: String,
    pub chunk_size: usize
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadEncryptedTableInput {
    pub database_id: String,
    pub table: String,
    pub encrypted_column: String,
    pub values: Vec<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadEncryptedTablePerUserInput {
    pub database_id: String,
    pub table: String,
    pub first_name: String,
    pub last_name: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHandleClientInput {
    pub database_id: String,
    pub opaque_handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clients {
    pub(crate) clients: Vec<String>,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }

    pub fn load() -> Result<Clients, Box<dyn std::error::Error>> {
        match klave::ledger::get_table(DATABASE_CLIENT_TABLE).get("ALL") {
            Ok(v) => {
                let clients: Clients = match serde_json::from_slice(&v) {
                    Ok(w) => w,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse client list: {}", e));
                        return Err(e.into());
                    }
                };
                Ok(clients)
            },
            Err(_e) => {
                let clients: Clients = Clients::new();
                Ok(clients)
            }
        }
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_clients = match serde_json::to_string(&self) {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to serialize database Clients: {}", e));
                return Err(e.into());
            }
        };
        klave::ledger::get_table(DATABASE_CLIENT_TABLE).set(&"ALL", &serialized_clients.as_bytes())
    }

    pub fn add(&mut self, db_input_details: DBInputDetails) -> Result<String, Box<dyn std::error::Error>> {
        let database_id = self.exists(&db_input_details).to_string();
        if database_id.is_empty() {
            let mut client = Client::new(
                db_input_details
            );
            client.save()?;
            self.clients.push(client.database_id.clone());
            self.save()?;
            Ok(client.database_id)
        } else {
            Ok(database_id)
        }
    }

    pub fn exists(&self, db_input_details: &DBInputDetails) -> String {
        for database_id in self.clients.iter() {
            if let Ok(client) = Client::load(database_id.to_string()) {
                if client.db_input_details.host == db_input_details.host && client.db_input_details.dbname == db_input_details.dbname
                && client.db_input_details.user == db_input_details.user && client.db_input_details.password == db_input_details.password {
                    return database_id.to_string();
                }
            }
        }
        String::new()
    }

    pub fn delete(&mut self, database_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(pos) = self.clients.iter().position(|x| x == database_id) {
            self.clients.remove(pos);
            klave::ledger::get_table(DATABASE_CLIENT_TABLE).remove(database_id)?;
            self.save()?;
            Ok(())
        } else {
            Err("Database ID not found".into())
        }
    }

    pub fn list(&self) -> Result<Vec<Client>, Box<dyn std::error::Error>> {
        let mut clients = Vec::new();
        for database_id in &self.clients {
            match Client::load(database_id.to_string()) {
                Ok(client) => clients.push(client),
                Err(e) => {
                    klave::notifier::send_string(&format!("Failed to load client {}: {}", database_id, e));
                }
            }
        }
        Ok(clients)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    database_id: String,
    db_input_details: DBInputDetails,
    opaque_handle: String,
    master_key_name: Option<String>, // Optional field for master key name
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")] // "type" is a reserved keyword in Rust, so we rename it
    pub field_type: u32,
    pub size: u64,
    pub scale: u32,
    pub nullable: bool,
    pub description: Option<String>, // Use Option<String> for nullable fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryClient {
    pub database_id: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostGreResponse<T> {
    pub fields: Vec<Field>,
    pub resultset: T, // Use Vec<Vec<Value>> for the varying resultset
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionDBDetails {
    pub id: String,
    pub encryption_key_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedQueryWithEncryptedUser {
    pub query: String,
    pub first_name_encryption: String,
    pub last_name_encryption: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedQueryWithEncryptedGender {
    pub query: String,
    pub gender_encryption: String
}

impl Client {

    pub fn new(
        db_input_details: DBInputDetails
    ) -> Self {
        let database_id = match klave::crypto::random::get_random_bytes(64).map(|x| hex::encode(x)) {
            Ok(id) => id,
            Err(e) => {
                klave::notifier::send_string(&format!("Failed to generate database ID: {}", e));
                String::new()
            }
        };
        Self {
            database_id: database_id,
            db_input_details: db_input_details,
            opaque_handle: String::new(),
            master_key_name: None,
        }
    }

    pub fn get_handle(&self) -> &str {
        &self.opaque_handle
    }

    // Loads a Client instance from the ledger using the database ID.
    pub fn load(database_id: String) -> Result<Client, Box<dyn std::error::Error>> {
        match klave::ledger::get_table(DATABASE_CLIENT_TABLE).get(&database_id) {
            Ok(v) => {
                let pgsql_client: Client = match serde_json::from_slice::<Client>(&v) {
                    Ok(w) => w,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to deserialize database Client: {}", e));
                        return Err(e.into());
                    }
                };
                Ok(pgsql_client)
            },
            Err(e) => Err(e.into())
        }
    }

    // Saves the master key.
    fn save_master_key(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create master key name
        let master_key_name = hex::encode(klave::crypto::random::get_random_bytes(32)?);
        // Generate master key
        let master_key = match generate_ecc_crypto_key()
        {
            Ok(key) => key,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to generate master key: {}", err));
                return Err(err);
            }
        };
        // Store the master key in the ledger
        let _ = match save_key(&master_key, &master_key_name) {
            Ok(_) => (),
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to save master key: {}", err));
                return Err(err.into());
            }
        };
        self.master_key_name = Some(master_key_name.clone());
        Ok(())
    }

    // Saves the Client instance to the ledger
    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        // Save master key
        self.save_master_key()?;
        // Serialize the Client instance to JSON
        let serialized = serde_json::to_string(self)?;

        // Store the serialized data in the ledger
        klave::ledger::get_table(DATABASE_CLIENT_TABLE).set(&self.database_id, serialized.as_bytes())?;

        Ok(())
    }

    // Constructs the PostgreSQL connection string from the DBInputDetails
    fn connection_string(&self) -> String {
        let mut conn_str = format!("host={} dbname={}", self.db_input_details.host, self.db_input_details.dbname);
        if !self.db_input_details.user.is_empty() {
            conn_str.push_str(&format!(" user={}", self.db_input_details.user));
        }
        if !self.db_input_details.password.is_empty() {
            conn_str.push_str(&format!(" password={}", self.db_input_details.password));
        }
        conn_str
    }

    // Connects to the PostgreSQL database using the connection string
    // and stores the opaque handle for further operations.
    pub fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        // Construct the PostgreSQL connection URI
        let uri = self.connection_string();

        // Open the PostgreSQL connection
        match klave::sql::connection_open(&uri) {
            Ok(opaque_handle) => {
                self.opaque_handle = opaque_handle;
                Ok(())
            }
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to connect to PostgreSQL: {}", err));
                Err(err.into())
            }
        }
    }

    // Queries the PostgreSQL database using the provided SQL query, returns a PostGreResponse.
    pub fn query<T>(&self, query: &str) -> Result<PostGreResponse<T>, Box<dyn std::error::Error>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {

        match klave::sql::query(&self.opaque_handle, query) {
            Ok(result) => {
                let response = match serde_json::from_str::<PostGreResponse<T>>(&result) {
                    Ok(res) => res,
                    Err(e) => {
                        klave::notifier::send_string(&format!("Failed to parse query result: {}", e));
                        return Err(e.into());
                    }
                };
                Ok(response)
            },
            Err(err) => {
                klave::notifier::send_string(&format!("Query failed: {}", err));
                Err(err.into())
            }
        }
    }

    // Executes a SQL command on the PostgreSQL database, returns the result as a String.
    pub fn execute(&self, query: &str) -> Result<String, Box<dyn std::error::Error>> {

        match klave::sql::execute(&self.opaque_handle, query) {
            Ok(result) => Ok(result),
            Err(err) => {
                klave::notifier::send_string(&format!("Execution failed: {}", err));
                Err(err.into())
            }
        }
    }

    // Encrypts the specified columns in the given DBTable.
    pub fn encrypt_columns(&mut self, db_table: DBTable) -> Result<(), Box<dyn std::error::Error>> {

        //for each column name, I retrieve both primary key + data associated to the column to encrypt
        for column in db_table.columns.clone() {
            let _ = match self.encrypt_single_column(column.clone(), &db_table) {
                Ok(_) => (),
                Err(err) => {
                    klave::notifier::send_string(&format!("Failed to encrypt column {}: {}", column, err));
                    return Err(err);
                }
            };
        }

        Ok(())
    }

    fn encrypt_single_column(&mut self, column: String, db_table: &DBTable) -> Result<(), Box<dyn std::error::Error>> {

        let table_name = &db_table.table;
        let chunk_size: usize = db_table.chunk_size;

        // Retrieve the primary key index and the columns to encrypt
        let answer: PostGreResponse<Vec<Vec<Value>>> = match self.get_column_to_encrypt(&db_table.primary_key, &db_table, &column)
        {
            Ok(column) => column,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to get columns to encrypt: {}", err));
                return Err(err);
            }
        };

        // Convert resultset
        let mut processed_rows: Vec<Vec<Value>> = answer.resultset;

        // Retrieve the master key
        let master_key_name = self.master_key_name.clone().ok_or("Master key name not set")?;
        let master_key = match klave::crypto::subtle::load_key(master_key_name.as_str()) {
            Ok(key) => key,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to load master key: {}", err));
                return Err(err);
            }
        };

        // Parse processed rows and encrypt specific column
        for row in processed_rows.iter_mut() {
            //the column to encrypt is the second one (index 1)
            let value = match row.get_mut(1) {
                Some (item) => {item},
                None => {
                    klave::notifier::send_string(&format!("Missing column: {}", column));
                    return Err(format!("Missing column: {}", column).into());
                }
            };

            let iv_encrypted_value = match encrypt_value(&master_key, table_name.to_string(), column.clone(), value.clone()) {
                Ok(enc_value) => enc_value,
                Err(err) => {
                    klave::notifier::send_string(&format!("Failed to encrypt value: {}", err));
                    return Err(err);
                }
            };

            //update the value with the encrypted value
            *value = serde_json::Value::String(iv_encrypted_value);
        }

        match self.update(processed_rows, answer.fields.clone(), table_name.clone(), chunk_size, column)
        {
            Ok(_) => {
                klave::notifier::send_string(&format!("Table {} successfully encrypted", table_name.clone()));
            },
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to update: {}", err));
                return Err(err);
            }
        };
        Ok(())
    }

    fn get_column_to_encrypt(&self, primary_key_field: &String, db_table: &DBTable, column: &String) -> Result<PostGreResponse<Vec<Vec<Value>>>, Box<dyn std::error::Error>> {

        // Build the query to retrieve the primary key and column to encrypt
        let query = format!("SELECT {},{} FROM {} ORDER BY {}", primary_key_field, column, db_table.table, primary_key_field);
        let result = match self.query::<Vec<Vec<Value>>>(&query) {
            Ok(response) => response,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to get the column to encrypt: {}", err));
                return Err(err);
            }
        };
        Ok(result)
    }

    fn update(&self, processed_rows: Vec<Vec<Value>>, fields: Vec<Field>, table: String, chunk_size: usize, column_name: String) -> Result<(), Box<dyn std::error::Error>> {
        if processed_rows.len() <= chunk_size {
            let query = self.build_update_query(processed_rows.clone(), fields, table.clone())?;
            // Execute the update
            let _ = match self.execute(&query)
            {
                Ok(_) => {
                    klave::notifier::send_string(&format!("Column {} of table {} has been encrypted", column_name, table));
                }
                Err(err) => {
                    klave::notifier::send_string(&format!("Failed to encrypt: {}", err));
                }
            };
        }
        else{
            let division_by_chunk:usize = processed_rows.len()/chunk_size;
            let remaining: usize = processed_rows.len() - division_by_chunk * chunk_size;

            for i in 0..division_by_chunk {
                let query = self.build_update_query(processed_rows[i*chunk_size..i*chunk_size+chunk_size].to_vec(), fields.clone(), table.clone())?;
                // Execute the update
                let _ = match self.execute(&query)
                {
                    Ok(_) => {
                        klave::notifier::send_string(&format!("Chunk {} of column {} of table {} has been encrypted", i, column_name, table));
                    }
                    Err(err) => {
                        klave::notifier::send_string(&format!("Failed to encrypt: {}", err));
                    }
                };
            }
            if remaining > 0 {
                let query = self.build_update_query(processed_rows[division_by_chunk * chunk_size..division_by_chunk * chunk_size+remaining].to_vec(), fields.clone(), table.clone())?;
                // Execute the update
                let _ = match self.execute(&query)
                {
                    Ok(_) => {
                        klave::notifier::send_string(&format!("Last chunk {} of column {} of table {} has been encrypted", division_by_chunk, column_name, table));
                    }
                    Err(err) => {
                        klave::notifier::send_string(&format!("Failed to encrypt: {}", err));
                    }
                };
            }
        }
        Ok(())
    }

    fn build_update_query(&self, processed_rows: Vec<Vec<Value>>, fields: Vec<Field>, table: String) -> Result<String, Box<dyn std::error::Error>> {

        // Iterate over the processed rows and build the update query
        if processed_rows.is_empty() {
            return Err("No rows to update".into());
        }
        // Primary key field
        let pk = &fields[0].name;
        // Retrieve the column names from the fields
        let column_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
        // All columns names
        let all_columns = column_names.join(",");
        // Build the update query
        let mut query = format!("WITH new_values ({}) AS (VALUES ", all_columns);
        // List all new values
        query.push_str(flatten_vec_of_vec_values_to_single_string(processed_rows).as_str());
        // Update
        query .push_str(&format!(") UPDATE {} SET ", table));
        // Update query
        for (i, column_name) in column_names.iter().enumerate() {
            if i==0 { continue; }
            query.push_str(&format!("{} = new_values.{}", column_name, column_name));
            if i < column_names.len() - 1 {
                query.push_str(", ");
            }
        };
        query.push_str(&format!(" FROM new_values WHERE {}.{} = new_values.{}", table, pk, pk));

        Ok(query)
    }

    pub fn build_encrypted_query(&self, input: ReadEncryptedTableInput) -> Result<String, Box<dyn std::error::Error>> {
        let table = input.table;
        let column = input.encrypted_column;
        let mut values = input.values;
        let mut query = "".to_string();

        // Retrieve the master key
        let master_key_name = self.master_key_name.clone().ok_or("Master key name not set")?;
        let master_key = match klave::crypto::subtle::load_key(master_key_name.as_str()) {
            Ok(key) => key,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to load master key: {}", err));
                return Err(err);
            }
        };

        for value in values.iter_mut() {
            // Reuse serde to be in line with encryption
            let serde_value = serde_json::Value::String(value.clone());

            let iv_encrypted_value = match encrypt_value(&master_key, table.clone(), column.clone(), serde_value.clone()) {
                Ok(enc_value) => enc_value,
                Err(err) => {
                    klave::notifier::send_string(&format!("Failed to encrypt value: {}", err));
                    return Err(err);
                }
            };
            //replace in value
            *value = iv_encrypted_value;
        }

        let list_values = format!(
            "({})",
            values.iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<String>>() // Collect directly into Vec<String>
                .join(",") // Join the collected Vec
        );

        query.push_str(&format!("SELECT * FROM {} WHERE {} in {}", table, column, list_values));

        Ok(query)
    }

    pub fn build_encrypted_query_per_user(&self, input: &ReadEncryptedTablePerUserInput) -> Result<EncryptedQueryWithEncryptedUser, Box<dyn std::error::Error>> {
        let table = &input.table;
        //retrieve first name and last name and trim them
        let first_name = &input.first_name.trim().to_string();
        let last_name = &input.last_name.trim().to_string();

        // Retrieve the master key
        let master_key_name = self.master_key_name.clone().ok_or("Master key name not set")?;
        let master_key = match klave::crypto::subtle::load_key(master_key_name.as_str()) {
            Ok(key) => key,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to load master key: {}", err));
                return Err(err);
            }
        };

        // Recompute first name encrypted value
        // Reuse serde to be in line with encryption
        let serde_value_first_name = serde_json::Value::String(first_name.clone());

        let iv_encrypted_value_first_name = match encrypt_value(&master_key, table.clone(), "first_name".to_string(), serde_value_first_name.clone()) {
            Ok(enc_value) => enc_value,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to encrypt value: {}", err));
                return Err(err);
            }
        };

        // Recompute first name encrypted value
        // Reuse serde to be in line with encryption
        let serde_value_last_name = serde_json::Value::String(last_name.clone());

        let iv_encrypted_value_last_name = match encrypt_value(&master_key, table.clone(), "last_name".to_string(), serde_value_last_name.clone()) {
            Ok(enc_value) => enc_value,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to encrypt value: {}", err));
                return Err(err);
            }
        };

        let query: String = format!("select u.first_name, u.last_name, pu.purchase_date, pr.product_name, pr.category, pr.brand, pr.description, pr.price from users as u \
            inner join purchases as pu on pu.user_id = u.id \
            inner join products as pr on pr.id = pu.product_id \
            where u.first_name = '{}' and u.last_name = '{}'",
            iv_encrypted_value_first_name,
            iv_encrypted_value_last_name);

        let res = EncryptedQueryWithEncryptedUser {
            query: query,
            first_name_encryption: iv_encrypted_value_first_name,
            last_name_encryption: iv_encrypted_value_last_name
        };

        Ok(res)
    }

    pub fn build_encrypted_query_per_gender(&self, gender: &String) -> Result<String, Box<dyn std::error::Error>> {
        // Retrieve the master key
        let master_key_name = self.master_key_name.clone().ok_or("Master key name not set")?;
        let master_key = match klave::crypto::subtle::load_key(master_key_name.as_str()) {
            Ok(key) => key,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to load master key: {}", err));
                return Err(err);
            }
        };

        // Encrypted query
        // Reuse serde to be in line with encryption
        let serde_value_gender = serde_json::Value::String(gender.to_string());

        let iv_encrypted_value_gender = match encrypt_value(&master_key, "users".to_string(), "gender".to_string(), serde_value_gender.clone()) {
            Ok(enc_value) => enc_value,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to encrypt value: {}", err));
                return Err(err);
            }
        };

        // Query
        let query = format!("SELECT avg(u.age) FROM users as u \
            INNER JOIN purchases AS pu ON pu.user_id = u.id \
            INNER JOIN products AS pr ON pr.id = pu.product_id \
            WHERE pu.total_price > 300 AND u.gender = '{}'", iv_encrypted_value_gender);

        Ok(query)
    }

}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::Value;

    use super::*;

    //Example of answer from the PostGreSql service
    #[test]
    fn test_deserialization() {
        let json_data = r#"
        {
            "fields": [
                {
                    "name": "product_id",
                    "type": 3,
                    "size": 18446744073709551615,
                    "scale": 0,
                    "nullable": true,
                    "description": null
                },
                {
                    "name": "name",
                    "type": 12,
                    "size": 104,
                    "scale": 0,
                    "nullable": true,
                    "description": null
                },
                {
                    "name": "price",
                    "type": 15,
                    "size": 655366,
                    "scale": 0,
                    "nullable": true,
                    "description": null
                }
            ],
            "resultset": [
                [
                    1,
                    "Laptop",
                    "1200.00"
                ],
                [
                    2,
                    "Mouse",
                    "25.50"
                ]
            ]
        }
        "#;

        let response: Vec<HashMap<String, Value>> = match serde_json::from_str::<PostGreResponse<Vec<Vec<Value>>>>(json_data) {
            Ok(res) => {
                let mut processed_rows: Vec<HashMap<String, Value>> = Vec::new();
                for row in res.resultset {
                    let mut processed_row = HashMap::new();
                    for (i, value) in row.into_iter().enumerate() {
                        let field_name = res.fields.get(i).map(|f| f.name.clone()).unwrap_or_default();
                        processed_row.insert(field_name, value);
                    }
                    processed_rows.push(processed_row);
                }
                processed_rows
            },
            Err(e) => {
                panic!("Failed to deserialize JSON: {}", e);
            }
        };

        // You can now access the data
        println!("{:?}", response);

        // Example of accessing fields
        // assert_eq!(response.fields.len(), 3);
        // assert_eq!(response.fields[0].name, "product_id");
        // assert_eq!(response.fields[0].field_type, 3);
        // assert_eq!(response.fields[0].description, None);

        // // Example of accessing resultset
        // assert_eq!(response.resultset.len(), 2);
        // assert_eq!(response.resultset[0][0], Value::from(1));
        // assert_eq!(response.resultset[0][1], Value::String("Laptop".to_string()));
        // assert_eq!(response.resultset[0][2], Value::String("1200.00".to_string()));

        if let Some(first_row_map) = response.first() {
            if let Some(product_id_value) = first_row_map.get("product_id") {
                if let Some(id) = product_id_value.as_i64() {
                    println!("\nExample access: Product ID of first row is {}", id);
                } else {
                    println!("\nExample access: Product ID of first row is not an integer: {:?}", product_id_value);
                }
            }
        }
    }
    #[test]
    fn test_usize() {
        let n: usize = 452;
        let chunk_size: usize = 100;
        let division: usize = n / chunk_size;
        assert_eq!(division, 4);
        let remaining: usize = n - division * chunk_size;
        assert_eq!(remaining, 52);
    }
}