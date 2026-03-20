use std::{collections::HashMap, str::FromStr};

#[derive(Default, Debug, Clone)]
pub struct ArgVals {
    simple_vals: Vec<String>,
    mapped_vals: HashMap<String, String>,
}

impl ArgVals {
    pub fn env() -> Self {
        let mut result = Self::default();
        for arg in std::env::args().skip(1) {
            match arg.find('=') {
                Some(eq) => result.add_mapped(&arg[..eq], &arg[eq+1..]),
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