use linguist::LanguageSet;

fn main() {
    let ls = LanguageSet::load_from_path("examples/gh-linguist/languages.yml").unwrap();
    println!("{:?}", &ls);

    println!("{:?}", &ls.resolve_language_by_extension(".ash"));
}
