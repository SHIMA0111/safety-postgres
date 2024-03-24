## SafetyPostgres
This library is for safety use of the PostgreSQL via Rust.  
Generally speaking, we need to take care the SQL injection when executing SQL statement.  
Almost all libraries has the protect mechanism like the parameter query.  
However, a developer needs to take care for more mechanisms validating the input values and so.  
By using this library crate, you can execute SQLs more safety.  
This library validates input values and generates parameterized SQL statement dynamically fitting to your input.   

**Of course, it is important that you pay attention for the risks. However, 
this library helps your application makes more safety!**

## Dependencies
 - `tokio = "1"`
   - A popular library for parallelized execution in Rust.
 - `tokio-postgres = "0.7"` with `with-chrono-0_4` feature
   - This library using tokio-postgres as the core of the executing SQL.
 - `chrono = "0.4"`
   - A library for treating datetime type in Rust
 - `serde = "1"` with `derive` feature
 - `serde_json = "0.4"`
   - These are used for Serialize and Deserialize the values to Json
 - `rust_decimal = "1"`
   - A library to support decimal format

## License
This project is licensed under the [MIT License](LICENSE-mit.md) and [Apache-2.0 License](LICENSE-ap.md)
