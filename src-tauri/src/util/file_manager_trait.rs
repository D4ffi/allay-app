use std::io::Error;
use std::path::PathBuf;

pub trait FileManager<T> {
    fn new(file_path: PathBuf) -> Self;
    fn load(&self) -> Result<T, Error>;
    fn save(&self, data: &T) -> Result<(), Error>;
    fn exists(&self) -> bool;
    fn create_default(&self) -> Result<(), Error>;
    fn get_file_path(&self) -> &PathBuf;
}

pub trait ConfigManager<T>: FileManager<T> {
    fn update_field(&self, key: &str, value: &str) -> Result<(), Error>;
    fn get_field(&self, key: &str) -> Result<String, Error>;
    fn reset_to_default(&self) -> Result<(), Error>;
}

pub trait CollectionManager<T, K>: FileManager<Vec<T>> {
    fn add_item(&self, item: T) -> Result<(), Error>;
    fn remove_item(&self, key: &K) -> Result<(), Error>;
    fn update_item(&self, key: &K, updated_item: T) -> Result<(), Error>;
    fn get_item(&self, key: &K) -> Result<Option<T>, Error>;
    fn get_all_items(&self) -> Result<Vec<T>, Error>;
    fn item_exists(&self, key: &K) -> Result<bool, Error>;
}

pub trait ServerConfigManager<T>: ConfigManager<T> {
    fn create_with_instance_path(&self, instance_path: &PathBuf) -> Result<(), Error>;
    fn validate_config(&self) -> Result<bool, Error>;
    fn backup_config(&self) -> Result<PathBuf, Error>;
    fn restore_from_backup(&self, backup_path: &PathBuf) -> Result<(), Error>;
}