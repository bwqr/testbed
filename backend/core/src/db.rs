use std::fmt::Debug;

use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::result::Error;
use diesel::sql_types::Text;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait DieselEnum: Debug + Sized {
    fn value(&self) -> String;

    fn status(input: &str) -> Result<Self, String>;

    fn build_from_string(row: String) -> Self;

    fn build_from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>;
}

impl<T> DieselEnum for T where T: Debug + Serialize + DeserializeOwned + Default {
    fn value(&self) -> String {
        serde_json::to_string(self).unwrap().replace("\"", "")
    }

    fn status(input: &str) -> Result<Self, String> {
        let input = String::from("\"") + input + "\"";

        serde_json::from_str(input.as_str())
            .map_err(|_| format!("Unexpected input {}, file: {}, line: {}", input, file!(), line!()))
    }

    fn build_from_string(row: String) -> Self {
        match Self::status(row.as_str()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{:?}", e);
                T::default()
            }
        }
    }

    fn build_from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        <Self as DieselEnum>::status(s.as_str())
            .map_err(|e| Box::new(Error::DeserializationError(e.into())) as Box<dyn std::error::Error + Send + Sync>)
    }
}
