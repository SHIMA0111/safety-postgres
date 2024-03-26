/// The `AppConfig` struct gets and holds configuration parameters for the application.
mod app_config;
/// This module contains utility functions for working with conditions.
pub mod conditions;
/// This module contains the custom error types used in the application.
pub mod errors;
/// The `generate_params` module provides functions for generating parameters using SQL execution.
mod generate_params;
/// Module `join_tables` provides functions and utilities for joining tables.
pub mod join_tables;
/// The `postgres_base` module contains the basic functionalities for interacting with a PostgreSQL database.
pub mod postgres;
/// This module contains the SQL base statement helper methods, which provides functionality for working with SQL databases.
pub mod sql_base;
/// Module `validators` provides functions for validating input data utilities.
mod validators;
pub mod json_parser;
mod converter;
mod format;
