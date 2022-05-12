#[allow(dead_code)]
use std::any::{TypeId};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

pub struct AlreadyExist;

impl Debug for AlreadyExist {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "AlreadyExists");
    }
}


pub trait ConfigurationProvider {}

pub struct Configuration {
    providers: HashMap<TypeId, Box<dyn Any>>,
}

impl Configuration {
    pub fn new() -> Self {
        let providers: Vec<Box<dyn ConfigurationProvider>> = Vec::new();
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn of<T: 'static + ConfigurationProvider>(&self) -> Option<T> {
        let t = TypeId::of::<T>();

        return match self.providers.contains_key(&t) {
            true => Some(self.providers.get(&t).unwrap().downcast_ref::<T>().unwrap().to_owned()),
            false => None
        }
    }

    pub fn len(&self) -> usize {
        return self.providers.len();
    }

    pub fn provide<T: 'static + ConfigurationProvider>(&mut self, provider: T) -> Result<&Self, AlreadyExist> {
        let t = TypeId::of::<T>();

        if self.providers.contains_key(&t) {
            return Err(AlreadyExist)
        }

        self.providers.insert(t, Box::new(provider));
        return Ok(self);
    }
}

pub struct BaseSettings {
    api_key: Option<String>,
    mpv_path: Option<String>,
    mpv_args: Option<String>
}

impl ConfigurationProvider for BaseSettings {}

impl Default for BaseSettings {
    fn default() -> Self {
        Self {
            api_key: None,
            mpv_path: Some("mpv".to_string()),
            mpv_args: Some("--no-video".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::BorrowMut;
    use std::cmp::Ordering;
    use crate::config::{BaseSettings, Configuration, ConfigurationProvider};

    #[test]
    fn is_providable() {
        let mut repo = Configuration::new();
        let mut config = repo.of::<BaseSettings>();
        assert_eq!(config.is_none(), true, "Assumed that empty configuration repo would return null value");
        assert_eq!(repo.len(), 0, "Assumed that repo would be empty got size of {}", repo.len());

        repo.provide(BaseSettings{
            api_key: Some("kake".to_string()),
            mpv_path: None,
            mpv_args: None
        }).expect("Expected no error");

        let mut config2 = repo.of::<BaseSettings>().unwrap().borrow_mut();

        assert_eq!(repo.len(), 1, "Assumed that repo would be empty got size of {}", repo.len());
        assert_eq!(config2.api_key.as_ref().unwrap().cmp(&"kake".to_string()), Ordering::Equal);
    }
}
