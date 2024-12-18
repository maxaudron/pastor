use rocket::figment::Figment;
use tera::Tera;

const BASE_TEMPLATE: &str = include_str!("../templates/base.html.tera");
const INDEX_TEMPLATE: &str = include_str!("../templates/index.html.tera");
const RETRIEVE_TEMPLATE: &str = include_str!("../templates/retrieve.html.tera");
const GUI_TEMPLATE: &str = include_str!("../templates/gui.html.tera");
const GUI_RESULT_TEMPLATE: &str = include_str!("../templates/gui_result.html.tera");
const DELETE_RESULT_TEMPLATE: &str = include_str!("../templates/delete_result.html.tera");

#[derive(serde::Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub storage_dir: String,
    pub database_dir: String,
    pub template_dir: Option<String>,
    pub deletion_interval_ms: u64,

    /// File for storing upload access tokens
    pub token_file: String,
}

impl AppConfig {
    pub fn new(src: &Figment) -> Self {
        let storage_dir = src
            .find_value("storage_dir")
            .expect("Could not find storage_dir config")
            .into_string()
            .unwrap();

        let database_dir = src
            .find_value("database_dir")
            .map_or(format!("{}/db", storage_dir), |x| x.into_string().unwrap());

        let token_file = src
            .find_value("database_dir")
            .map_or(format!("{}/tokens.toml", database_dir), |x| x.into_string().unwrap());

        let template_dir = src
            .find_value("database_dir")
            .map_or(None, |x| Some(x.into_string().unwrap()));

        let deletion_interval_ms = src
            .find_value("deletion_interval_ms")
            .map_or(3_600_000, |x| {
                x.into_string().unwrap().parse().expect("failed to parse deletion_interval_ms")
            });

        AppConfig {
            storage_dir,
            database_dir,
            template_dir,
            deletion_interval_ms,
            token_file,
        }
    }
}

pub async fn setup_templates(config: &AppConfig) -> Tera {
    let mut tera = match config.template_dir.as_ref() {
        Some(s) => {
            let mut tera = Tera::parse(&format!("{}/*", s)).unwrap();
            println!("Using external templates at {}", s);
            tera.add_template_files(vec![
                (format!("{}/base.html.tera", s), Some("base")),
                (format!("{}/index.html.tera", s), Some("index")),
                (format!("{}/retrieve.html.tera", s), Some("retrieve")),
                (format!("{}/gui.html.tera", s), Some("gui")),
                (format!("{}/gui_result.html.tera", s), Some("gui_result")),
                (
                    format!("{}/delete_result_result.html.tera", s),
                    Some("delete_result"),
                ),
            ])
            .unwrap();
            tera
        }
        None => {
            let mut tera = Tera::parse("/templates/*").unwrap();
            println!("Using embedded templates");
            tera.add_raw_templates(vec![
                ("base", BASE_TEMPLATE),
                ("index", INDEX_TEMPLATE),
                ("retrieve", RETRIEVE_TEMPLATE),
                ("gui", GUI_TEMPLATE),
                ("gui_result", GUI_RESULT_TEMPLATE),
                ("delete_result", DELETE_RESULT_TEMPLATE),
            ])
            .unwrap();
            tera
        }
    };

    tera.build_inheritance_chains().unwrap();
    return tera;
}
