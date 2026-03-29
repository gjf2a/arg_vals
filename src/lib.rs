use std::{collections::HashMap, fmt::Display, str::FromStr};

pub fn arg_param(arg: &str) -> Option<(&str, &str)> {
    arg.find('=').map(|eq| (&arg[..eq], &arg[eq+1..]))
}

#[derive(Default, Debug, Clone)]
pub struct ArgVals {
    simple_vals: Vec<String>,
    mapped_vals: HashMap<String, String>,
}

impl ArgVals {
    pub fn env() -> Self {
        let mut result = Self::default();
        for arg in std::env::args().skip(1) {
            match arg_param(&arg) {
                Some((arg, value)) => result.add_mapped(arg, value),
                None => result.add_simple(&arg),
            }
        }
        result
    }

    pub fn add_simple(&mut self, simple: &str) {
        self.simple_vals.push(simple.to_string());
    }

    pub fn add_mapped(&mut self, key: &str, value: &str) {
        self.mapped_vals.insert(key.to_string(), value.to_string());
    }

    pub fn len(&self) -> usize {
        self.simple_vals.len() + self.mapped_vals.len()
    }

    pub fn num_symbols(&self) -> usize {
        self.simple_vals.len()
    }

    pub fn get_symbol(&self, i: usize) -> &str {
        self.simple_vals[i].as_str()
    }

    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.simple_vals.iter().any(|s| s == symbol)
    }

    pub fn key_value_pairs<N: Copy + FromStr>(&self) -> impl Iterator<Item = (&str, N)> {
        self.mapped_vals
            .iter()
            .filter_map(|(k, v)| v.parse::<N>().map(|v| (k.as_str(), v)).ok())
    }
    
    pub fn get_str_value(&self, key: &str) -> Option<&String> {
        self.mapped_vals.get(key)
    }

    pub fn get_value<N: Copy + FromStr>(&self, key: &str) -> Option<N> {
        self.mapped_vals
            .get(key)
            .map(|v| v.parse::<N>())
            .and_then(|v| v.ok())
    }

    pub fn get_duple<N: Copy + FromStr>(&self, key: &str) -> Option<(N, N)> {
        self.mapped_vals.get(key).and_then(|v| {
            let values = v.split(",").collect::<Vec<_>>();
            if values.len() == 2 {
                match values[0].parse::<N>() {
                    Err(_) => None,
                    Ok(v1) => match values[1].parse::<N>() {
                        Err(_) => None,
                        Ok(v2) => Some((v1, v2)),
                    },
                }
            } else {
                None
            }
        })
    }
}

pub struct ArgDocs {
    executable_name: String,
    arg_defs: Vec<(String,String,Option<String>)>,
}

impl ArgDocs {
    pub fn new(executable_name: &str, defs: &Vec<(&str,&str,&str)>) -> Self {
        Self {
            executable_name: executable_name.to_string(),
            arg_defs: defs.iter().map(|(arg, arg_type, def)| (arg.to_string(), arg_type.to_string(), if def.len() == 0 {None} else {Some(def.to_string())})).collect(),
        }
    }
}

macro_rules! write_opt {
    ($f:tt, $s:expr, $sp:expr) => {
        $s.as_ref().map_or(Ok(()), |s| write!($f, "{}{s}", $sp))
    };
}

impl Display for ArgDocs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Usage: {}", self.executable_name)?;
        for (sym, def, sym_type) in self.arg_defs.iter().filter(|(arg,_,_)| arg_param(arg).is_none()) {
            write!(f, " {sym} ({def})")?;
            write_opt!(f, sym_type, ' ')?;
            write!(f, " ")?;
        }
        for (assign, def, assign_type) in self.arg_defs.iter().filter(|(arg,_,_)| arg_param(arg).is_some()) {
            write!(f, "\n\t{assign} ({def})")?;
            write_opt!(f, assign_type, "\n\t\t")?;
        }
        Ok(())
    }
}
