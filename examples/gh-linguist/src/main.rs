use linguist::LanguageSet;

fn main() {
    let ldp = std::env::var("LANGUAGE_DEF_PATH").expect("cannot find env `LANGUAGE_DEF_PATH`");
    let ls = LanguageSet::load_from_path(ldp).unwrap();

    println!("{:?}", ls.resolve_languages_by_extension(".vhost"));
    println!("---");
    println!("{:?}", ls.resolve_languages_by_filename("Dockerfile"));
}
