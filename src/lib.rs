use std::{collections::HashMap, fmt::Display, str::FromStr};

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
                Err(_) => Err(anyhow::anyhow!("Error parsing {str_value}")),
            }
        } else {
            Err(anyhow::anyhow!("{key} missing"))
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

pub struct ArgDocs {
    executable_name: String,
    arg2type_default: HashMap<String, (String, Option<String>)>,
}

pub fn merged_arg_docs<'a, A: Iterator<Item = &'a ArgDocs>>(
    executable_name: &str,
    arg_docs: A,
) -> ArgDocs {
    let mut result = ArgDocs::new(executable_name, &vec![]);
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

    pub fn get_args_with_defaults(&self) -> ArgVals {
        let mut result = ArgVals::env();
        for (arg, (_, def)) in self.arg2type_default.iter() {
            if assignment_param(arg).is_some() && result.get_str_value(arg).is_err() {
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
            write!(f, "\t\n{assign}={assign_type}")?;
            if let Some(d) = def {
                write!(f, " [{d}]")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
