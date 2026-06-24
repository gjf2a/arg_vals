use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    str::FromStr,
};

pub fn assignment_param(arg: &str) -> Option<(&str, &str)> {
    arg.find('=').map(|eq| (&arg[..eq], &arg[eq + 1..]))
}

#[derive(Default, Debug, Clone)]
pub struct ArgVals {
    mapped_vals: HashMap<String, String>,
}

impl ArgVals {
    pub fn env() -> Self {
        let mut result = Self::default();
        for arg in std::env::args().skip(1) {
            if let Some((arg, value)) = assignment_param(&arg) {
                result.add_mapping(arg, value);
            }
        }
        result
    }

    pub fn add_mapping(&mut self, key: &str, value: &str) {
        self.mapped_vals.insert(key.to_string(), value.to_string());
    }

    pub fn len(&self) -> usize {
        self.mapped_vals.len()
    }

    pub fn key_value_pairs<N: Copy + FromStr>(&self) -> impl Iterator<Item = (&str, N)> {
        self.mapped_vals
            .iter()
            .filter_map(|(k, v)| v.parse::<N>().map(|v| (k.as_str(), v)).ok())
    }

    pub fn get_str_value(&self, key: &str) -> anyhow::Result<&String> {
        self.mapped_vals
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("{key} missing"))
    }

    pub fn get_value<N: Copy + FromStr>(&self, key: &str) -> anyhow::Result<N> {
        if let Some(str_value) = self.mapped_vals.get(key) {
            match str_value.parse::<N>() {
                Ok(n) => Ok(n),
                Err(_) => anyhow::bail!("Error parsing {str_value}"),
            }
        } else {
            anyhow::bail!("{key} missing")
        }
    }

    pub fn get_optional_value<N: Copy + FromStr>(&self, key: &str) -> anyhow::Result<Option<N>> {
        if let Some(str_value) = self.mapped_vals.get(key) {
            match str_value.parse::<N>() {
                Ok(n) => Ok(Some(n)),
                Err(_) => anyhow::bail!("Error parsing {str_value}"),
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_duple<N: Copy + FromStr>(&self, key: &str) -> anyhow::Result<(N, N)> {
        match self.mapped_vals.get(key) {
            Some(v) => {
                let values = v.split(",").collect::<Vec<_>>();
                if values.len() == 2 {
                    match values[0].parse::<N>() {
                        Err(_) => Err(anyhow::anyhow!("Error when parsing {}", values[0])),
                        Ok(v1) => match values[1].parse::<N>() {
                            Err(_) => Err(anyhow::anyhow!("Error when parsing {}", values[1])),
                            Ok(v2) => Ok((v1, v2)),
                        },
                    }
                } else {
                    Err(anyhow::anyhow!(
                        "Error parsing {v} as a point; need exactly 2 elements"
                    ))
                }
            }
            None => Err(anyhow::anyhow!("No value for {key}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArgDocs {
    executable_name: String,
    arg2type_default: BTreeMap<String, (String, Option<String>)>,
}

pub fn merged_arg_docs<'a, A: Iterator<Item = &'a ArgDocs>>(arg_docs: A) -> ArgDocs {
    let executable_name = std::env::args().next().unwrap();
    let mut result = ArgDocs::new(&executable_name, &vec![]);
    for arg_doc in arg_docs {
        for (arg, (arg_type, arg_default)) in arg_doc.arg2type_default.iter() {
            match result.arg2type_default.get_mut(arg) {
                None => {
                    result.arg2type_default.insert(
                        arg.to_string(),
                        (
                            arg_type.to_string(),
                            arg_default.as_ref().map(|s| s.to_string()),
                        ),
                    );
                }
                Some((_, prev_default)) => {
                    if prev_default.is_none() {
                        *prev_default = arg_default.as_ref().map(|s| s.clone());
                    }
                }
            }
        }
    }
    result
}

impl ArgDocs {
    pub fn new(executable_name: &str, defs: &Vec<(&str, &str, &str)>) -> Self {
        Self {
            executable_name: executable_name.to_string(),
            arg2type_default: defs
                .iter()
                .map(|(arg, arg_type, def)| {
                    (
                        arg.to_string(),
                        (
                            arg_type.to_string(),
                            if def.len() == 0 {
                                None
                            } else {
                                Some(def.to_string())
                            },
                        ),
                    )
                })
                .collect(),
        }
    }

    pub fn set_default(&mut self, arg: &str, arg_default: &str) -> anyhow::Result<()> {
        match self.arg2type_default.get_mut(arg) {
            None => Err(anyhow::anyhow!("Missing argument: {arg}")),
            Some((_, current)) => {
                *current = Some(arg_default.to_string());
                Ok(())
            }
        }
    }

    pub fn get_args_with_defaults(&self) -> ArgVals {
        let mut result = ArgVals::env();
        for (arg, (_, def)) in self.arg2type_default.iter() {
            if result.get_str_value(arg).is_err() {
                if let Some(default_def) = def {
                    result.add_mapping(arg, default_def);
                }
            }
        }
        result
    }
}

impl Display for ArgDocs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Usage: {}", self.executable_name)?;
        for (assign, (assign_type, def)) in self.arg2type_default.iter() {
            write!(f, "\n\t{assign}={assign_type}")?;
            if let Some(d) = def {
                write!(f, " [{d}]")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{ArgDocs, ArgVals};
    use map_macro::hash_map;

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    enum Tester {
        Test1,
        Test2,
    }

    impl FromStr for Tester {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "Test1" => Ok(Self::Test1),
                "Test2" => Ok(Self::Test2),
                _ => anyhow::bail!("Unrecognized alternative: {s}"),
            }
        }
    }

    #[test]
    fn test_arg_vals() {
        let arg_vals = ArgVals {
            mapped_vals: hash_map! {
                "--meters-per-cell".to_string() => "0.1".to_string(),
                "--robot".to_string() => "archangel".to_string(),
                "--num-particles".to_string() => "1000".to_string(),
                "--save-map".to_string() => "true".to_string()
            },
        };
        assert_eq!(arg_vals.get_value::<f64>("--meters-per-cell").unwrap(), 0.1);
        assert_eq!(arg_vals.get_str_value("--robot").unwrap(), "archangel");
        assert_eq!(arg_vals.get_value::<bool>("--save-map").unwrap(), true);
        assert_eq!(
            arg_vals.get_value::<usize>("--num-particles").unwrap(),
            1000
        );
    }

    #[test]
    fn test_arg_docs() {
        let arg_docs = ArgDocs::new(
            "bit_slam_node",
            &vec![
                ("--robot", "str", "archangel"),
                ("--num-particles", "usize", "1000"),
                ("--meters-per-cell", "f64", "0.1"),
                ("--save-map", "bool", "true"),
                ("--tester", "Tester", "Test1"),
                ("--period", "Option<usize>", "100"),
            ],
        );

        let vals = arg_docs.get_args_with_defaults();
        assert_eq!(vals.get_str_value("--robot").unwrap(), "archangel");
        assert_eq!(vals.get_value::<usize>("--num-particles").unwrap(), 1000);
        assert_eq!(vals.get_value::<f64>("--meters-per-cell").unwrap(), 0.1);
        assert_eq!(vals.get_value::<bool>("--save-map").unwrap(), true);
        assert_eq!(vals.get_value::<Tester>("--tester").unwrap(), Tester::Test1);
        assert_eq!(vals.get_optional_value::<usize>("--period").unwrap(), Some(100));
        assert_eq!(vals.get_optional_value::<usize>("--limit").unwrap(), None);
    }
}
