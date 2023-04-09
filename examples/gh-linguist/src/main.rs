use linguist::{
    github::{load_github_linguist_heuristics, load_github_linguist_languages},
    resolver::{
        resolve_language_by_content, resolve_languages_by_extension, resolve_languages_by_filename,
        InMemoryLanguageContainer,
    },
};

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

    match resolve_languages_by_filename(&args[1], &lc) {
        Ok(result) => println!(
            "Possible languages by filename: {}",
            result
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>()
                .join(", ")
        ),
        _ => println!("No result by `resolve_languages_by_filename()`"),
    };

    match resolve_languages_by_extension(&args[1], &lc) {
        Ok(result) => println!(
            "Possible languages by extension: {}",
            result
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>()
                .join(", ")
        ),
        _ => println!("No result by `resolve_languages_by_extension()`"),
    };

    match resolve_language_by_content(&args[1], &lc) {
        Ok(result) => match result {
            Some(lang) => println!("Language by content: {}", lang.name),
            _ => println!("No heuristic found"),
        },
        _ => println!("No result by `resolve_languages_by_content()`"),
    };
}
