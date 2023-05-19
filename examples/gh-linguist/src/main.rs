use linguist::{
    resolver::{resolve_language, InMemoryLanguageContainer, Scope},
    utils::{is_configuration, is_documentation, is_dotfile, is_vendor},
};
use regex::RegexSet;
use std::{collections::HashMap, fmt::Display, os::unix::prelude::MetadataExt, path::Path};
use walkdir::WalkDir;

pub mod predefined {
    include!(concat!(env!("OUT_DIR"), "/languages.rs"));
    include!(concat!(env!("OUT_DIR"), "/heuristics.rs"));
    include!(concat!(env!("OUT_DIR"), "/vendors.rs"));
    include!(concat!(env!("OUT_DIR"), "/documentation.rs"));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut lc = InMemoryLanguageContainer::default();
    for lang in predefined::languages() {
        lc.register_language(lang);
    }

    for h in predefined::heuristics() {
        for ext in &h.extensions {
            lc.register_heuristic_rule(ext.clone(), h.clone());
        }
    }

    let root = Path::new(&args[1]);
    if !root.is_dir() {
        eprintln!("path isn't a directory");
        return;
    }

    let mut breakdown = LanguageBreakdown {
        usages: HashMap::new(),
        total_size: 0,
    };

    // todo: this hashmap is currently useless, it may be used as an alternative way to get the
    // breakdown of all considered files...
    // let mut stats: HashMap<String, Vec<String>> = HashMap::new();

    let rules = RegexSet::new(predefined::VENDORS).unwrap();
    let docs = RegexSet::new(predefined::DOCUMENTATION).unwrap();

    let walker = WalkDir::new(root);
    for entry in walker.into_iter().flatten() {
        if entry.path().is_dir() {
            continue;
        }

        let relative_path = entry.path().strip_prefix(root).unwrap();
        if is_vendor(entry.path(), &rules)
            || is_documentation(relative_path, &docs)
            || is_dotfile(relative_path)
            || is_configuration(relative_path)
        {
            continue;
        }

        let language = match resolve_language(entry.path(), &lc) {
            Ok(Some(lang)) => lang,
            _ => continue,
        };

        if language.scope != Scope::Programming && language.scope != Scope::Markup {
            continue;
        }

        // stats
        //     .entry(language.name.clone())
        //     .or_insert_with(Vec::new)
        //     .push(entry.path().display().to_string());
        breakdown.add_usage(&language.name, entry.metadata().unwrap().size());
    }
    println!("{}", breakdown);
}

pub struct LanguageBreakdown {
    usages: HashMap<String, u64>,
    total_size: u64,
}

impl LanguageBreakdown {
    pub fn add_usage(&mut self, lang: &str, size: u64) {
        let entry = self.usages.entry(lang.to_string()).or_insert(size);
        *entry += size;

        self.total_size += size;
    }
}

impl Display for LanguageBreakdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut values: Vec<(&String, &u64)> = self.usages.iter().collect();
        values.sort_by_key(|&(_, v)| v);
        values.reverse();

        for (lang, size) in values {
            let percentage = ((*size as f64) * 100.0) / (self.total_size as f64);
            let _ = writeln!(f, "{:-6.2}% {:-7}   {}", percentage, size, lang);
        }

        Ok(())
    }
}
