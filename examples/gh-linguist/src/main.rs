use linguist::{
    github::{load_github_linguist_heuristics, load_github_linguist_languages},
    resolver::{resolve_language, InMemoryLanguageContainer},
};
use std::{collections::HashMap, fmt::Display, os::unix::prelude::MetadataExt, path::Path};
use walkdir::WalkDir;

fn main() {
    let ldp = std::env::var("LANGUAGE_DEF_PATH").expect("cannot find env `LANGUAGE_DEF_PATH`");
    let lhp = std::env::var("HEURISTIC_DEF_PATH").expect("cannot find env `HEURISTIC_DEF_PATH`");
    let args: Vec<String> = std::env::args().collect();

    let languages = load_github_linguist_languages(ldp).unwrap();
    let heuristics = load_github_linguist_heuristics(lhp).unwrap();

    let mut lc = InMemoryLanguageContainer::default();
    for lang in languages {
        lc.register_language(lang);
    }

    for h in heuristics {
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

    let walker = WalkDir::new(root);
    for entry in walker.into_iter().flatten() {
        if entry.path().is_dir() {
            continue;
        }

        let language = if let Ok(lang) = resolve_language(entry.path(), &lc) {
            lang.unwrap()
        } else {
            continue;
        };

        if language.scope != Scope::Programming && language.scope != Scope::Markup {
            continue;
        }

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
