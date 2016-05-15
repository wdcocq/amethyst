
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::fmt;

use yaml_rust::{Yaml, YamlLoader};

pub enum ConfigError {
    YamlParse(String),
    FileNotFound(String),
}

impl ConfigError {
    pub fn to_string(&self) -> String {
        match self {
            &ConfigError::YamlParse(ref path) => format!("Failed to parse YAML object: {} ", path),
            &ConfigError::FileNotFound(ref path) => format!("File was not found: {}", path),
        }
    }
}

impl fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub trait FromYaml: Sized {
    /// Convert yaml element into a rust type,
    /// Raises an error if it is not the yaml element expected
    fn from_yaml(&Yaml) -> Result<Self, ConfigError>;
}

macro_rules! yaml_int {
    ($t:ty) => {
        impl FromYaml for $t {
            fn from_yaml(config: &Yaml) -> Result<Self, ConfigError> {
                let num: $t = try!(config.as_i64()
                    .ok_or(ConfigError::YamlParse("expect integer".to_string()))) as $t;
                Ok(num)
            }
        }
    }
}

yaml_int!(i8);
yaml_int!(i16);
yaml_int!(i32);
yaml_int!(i64);
yaml_int!(u8);
yaml_int!(u16);
yaml_int!(u32);
yaml_int!(u64);

impl FromYaml for f32 {
    fn from_yaml(config: &Yaml) -> Result<Self, ConfigError> {
        Ok(try!(config.as_f64().ok_or(ConfigError::YamlParse("expect float".to_string()))) as f32)
    }
}

impl FromYaml for f64 {
    fn from_yaml(config: &Yaml) -> Result<Self, ConfigError> {
        Ok(try!(config.as_f64().ok_or(ConfigError::YamlParse("expect float".to_string()))))
    }
}

impl FromYaml for bool {
    fn from_yaml(config: &Yaml) -> Result<Self, ConfigError> {
        Ok(try!(config.as_bool().ok_or(ConfigError::YamlParse("expect boolean".to_string()))))
    }
}

impl FromYaml for String {
    fn from_yaml(config: &Yaml) -> Result<Self, ConfigError> {
        if let &Yaml::String(ref string) = config {
            Ok(string.clone())
        } else {
            Err(ConfigError::YamlParse("expect string".to_string()))
        }
    }
}

impl FromYaml for () {
    fn from_yaml(config: &Yaml) -> Result<Self, ConfigError> {
        if config.is_null() {
            Ok(())
        } else {
            Err(ConfigError::YamlParse("expect null".to_string()))
        }
    }
}

macro_rules! yaml_array {
    ($n:expr => $($i:expr)+) => {
        impl<T: FromYaml> FromYaml for [T;$n] {
            fn from_yaml(config: &Yaml) -> Result<Self, ConfigError> {
                if let &Yaml::Array(ref array) = config {
                    if array.len() != $n {
                        return Err(ConfigError::YamlParse(format!("expect list of length {}",$n)));
                    }

                    let mut arr = array.clone();

                    Ok([
                       $(
                            try!(T::from_yaml(&arr.remove(0))
                                .map_err(|e| ConfigError::YamlParse(format!("list[{}]: {:?}", $i, e)))),
                       )+
                    ])
                } else {
                    Err(ConfigError::YamlParse("expect list".to_string()))
                }
            }
        }
    }
}

yaml_array!(1 => 0);
yaml_array!(2 => 0 1);
yaml_array!(3 => 0 1 2);
yaml_array!(4 => 0 1 2 3);
yaml_array!(5 => 0 1 2 3 4);
yaml_array!(6 => 0 1 2 3 4 5);
yaml_array!(7 => 0 1 2 3 4 5 6);
yaml_array!(8 => 0 1 2 3 4 5 6 7);
yaml_array!(9 => 0 1 2 3 4 5 6 7 8);
yaml_array!(10 => 0 1 2 3 4 5 6 7 8 9);

macro_rules! config {
    ($root:ident {
        $( $field:ident: $ty:ty = $name:expr, )*
    }) => {

        #[derive(Clone, Debug)]
        pub struct $root {
            $( pub $field: $ty, )*
        }

        impl $root {
            pub fn default() -> $root {
                $root {
                    $( $field: $name, )*
                }
            }

            // path should be the root config.yml if separated into multiple files
            pub fn from_file(path: &Path) -> Result<$root, ConfigError> {
                let mut file = try!(File::open(path).map_err(|e| ConfigError::FileNotFound(path.to_str().unwrap().to_owned())));
                let mut buffer = String::new();
                file.read_to_string(&mut buffer);

                let yaml = YamlLoader::load_from_str(&buffer).unwrap();
                let hash = &yaml[0];

                $root::from_yaml(hash)
            }

            fn name(&self) -> &'static str {
                stringify!($root)
            }
        }

        impl FromYaml for $root {
            fn from_yaml(config: &Yaml) -> Result<Self, ConfigError> {
                let default = $root::default();

                Ok($root {
                    $(
                        $field: {
                            let val = <$ty>::from_yaml(&config[stringify!($field)]);

                            match val {
                                Ok(found) => found,
                                Err(e) => {
                                    let err = match e {
                                        ConfigError::YamlParse(err) => err,
                                        _ => "Unknown Parsing Error".to_string(),
                                    };

                                    let error = ConfigError::YamlParse(format!("{}->{}: {}", stringify!($root), stringify!($field), err));

                                    println!("{:?}", error);
                                    default.$field
                                },
                            }
                        },
                    )*
                })
            }
        }
    }
}

// Defines types along with defaulting values
config!(DisplayConfig {
    brightness: f64 = 1.0,
    fullscreen: bool = false,
    size: [u16; 2] = [1024, 768],
});

config!(LoggingConfig {
    file_path: String = "new_project.log".to_string(),
    output_level: String = "warn".to_string(),
    logging_level: String = "debug".to_string(),
});

config!(Config {
    display: DisplayConfig = DisplayConfig::default(),
    logging: LoggingConfig = LoggingConfig::default(),
});