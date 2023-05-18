use linguist_build::DefinitionSet;

fn main() {
    linguist_build::Config::new()
        .add_definition(DefinitionSet {
            language_url: Some(linguist_build::GITHUB_LINGUIST_LANGUAGES_URL.to_string()),
            heuristics_url: Some(linguist_build::GITHUB_LINGUIST_HEURISTICS_URL.to_string()),
            vendor_url: Some(linguist_build::GITHUB_LINGUIST_VENDOR_URL.to_string()),
            out_name: "github_linguist".to_string(),
        })
        .generate(); 
}
