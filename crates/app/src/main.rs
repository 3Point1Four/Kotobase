use std::path::{Path, PathBuf};

use eframe::egui;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

const APPLICATION_NAME: &str = "Kotobase";

const LOGO_PATH: &str = "assets/kotobase_original.png";
const WHITE_LOGO_PATH: &str = "assets/kotobase_white.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;

    let database_path = runtime.block_on(initialize())?;

    launch_ui(database_path)
}

async fn initialize() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let database_path = database_path()?;

    println!("Starting {APPLICATION_NAME}...");
    println!("Database: {}", database_path.display());

    let options = SqliteConnectOptions::new()
        .filename(&database_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    storage::initialize_database(&pool).await?;
    verify_database(&pool).await?;

    println!("Database initialized.");

    drop(pool);

    Ok(database_path)
}

fn launch_ui(
    database_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo_path = project_path(LOGO_PATH);
    let white_logo_path = project_path(WHITE_LOGO_PATH);

    let icon_data = load_window_icon(&logo_path)?;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APPLICATION_NAME)
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(icon_data),
        ..Default::default()
    };

    eframe::run_native(
        APPLICATION_NAME,
        native_options,
        Box::new(move |creation_context| {
            Ok(Box::new(KotobaseApp::new(
                creation_context,
                database_path,
                logo_path,
                white_logo_path,
            )))
        }),
    )?;

    Ok(())
}

struct KotobaseApp {
    selected_section: Section,
    database_path: PathBuf,
    logo: egui::TextureHandle,
    white_logo: egui::TextureHandle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Section {
    #[default]
    Home,
    Vocabulary,
    Kanji,
    Grammar,
    References,
}

impl Section {
    fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Vocabulary => "Vocabulary",
            Self::Kanji => "Kanji",
            Self::Grammar => "Grammar",
            Self::References => "References",
        }
    }
}

impl KotobaseApp {
    fn new(
        context: &eframe::CreationContext<'_>,
        database_path: PathBuf,
        logo_path: PathBuf,
        white_logo_path: PathBuf,
    ) -> Self {
        let logo = load_texture(
            &context.egui_ctx,
            "kotobase_logo",
            &logo_path,
        );

        let white_logo = load_texture(
            &context.egui_ctx,
            "kotobase_white_logo",
            &white_logo_path,
        );

        Self {
            selected_section: Section::default(),
            database_path,
            logo,
            white_logo,
        }
    }

    fn navigation_button(
        &mut self,
        ui: &mut egui::Ui,
        section: Section,
        label: &str,
    ) {
        let selected =
            self.selected_section == section;

        if ui
            .selectable_label(selected, label)
            .clicked()
        {
            self.selected_section = section;
        }
    }

    fn home(&self, ui: &mut egui::Ui) {
        ui.heading("Welcome to Kotobase");

        ui.add_space(8.0);

        ui.label(
            "Your Japanese language workspace.",
        );

        ui.add_space(32.0);

        ui.horizontal(|ui| {
            self.home_card(
                ui,
                "Vocabulary",
                "Words, readings, meanings, and parts of speech.",
            );

            self.home_card(
                ui,
                "Kanji",
                "Characters, readings, meanings, and metadata.",
            );

            self.home_card(
                ui,
                "References",
                "Connections between everything in your library.",
            );
        });
    }

    fn home_card(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        description: &str,
    ) {
        egui::Frame::group(ui.style()).show(
            ui,
            |ui| {
                ui.set_min_size(
                    egui::vec2(230.0, 130.0),
                );

                ui.heading(title);

                ui.add_space(8.0);

                ui.label(description);
            },
        );
    }

    fn placeholder(
        &self,
        ui: &mut egui::Ui,
        title: &str,
    ) {
        ui.heading(title);

        ui.add_space(12.0);

        ui.label(
            "This section is connected to the application architecture.",
        );

        ui.add_space(4.0);

        ui.label(
            "The actual workspace for this section will be implemented next.",
        );
    }
}

impl eframe::App for KotobaseApp {
    fn update(
        &mut self,
        context: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::TopBottomPanel::top("top_bar").show(
            context,
            |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::new(
                            &self.logo,
                        )
                        .fit_to_exact_size(
                            egui::vec2(32.0, 32.0),
                        ),
                    );

                    ui.add_space(8.0);

                    ui.heading(APPLICATION_NAME);

                    ui.separator();

                    ui.label(
                        egui::RichText::new(
                            self.selected_section.title(),
                        )
                        .weak(),
                    );
                });
            },
        );

        egui::SidePanel::left("navigation")
            .resizable(false)
            .default_width(190.0)
            .show(context, |ui| {
                ui.add_space(18.0);

                ui.vertical_centered(|ui| {
                    ui.add(
                        egui::Image::new(
                            &self.white_logo,
                        )
                        .fit_to_exact_size(
                            egui::vec2(110.0, 110.0),
                        ),
                    );

                    ui.add_space(8.0);

                    ui.label(
                        egui::RichText::new(
                            APPLICATION_NAME,
                        )
                        .strong(),
                    );
                });

                ui.add_space(22.0);

                ui.separator();

                ui.add_space(12.0);

                ui.heading("Library");

                ui.add_space(12.0);

                self.navigation_button(
                    ui,
                    Section::Home,
                    "Home",
                );

                self.navigation_button(
                    ui,
                    Section::Vocabulary,
                    "Vocabulary",
                );

                self.navigation_button(
                    ui,
                    Section::Kanji,
                    "Kanji",
                );

                self.navigation_button(
                    ui,
                    Section::Grammar,
                    "Grammar",
                );

                ui.add_space(12.0);

                ui.separator();

                ui.add_space(12.0);

                self.navigation_button(
                    ui,
                    Section::References,
                    "References",
                );

                ui.with_layout(
                    egui::Layout::bottom_up(
                        egui::Align::LEFT,
                    ),
                    |ui| {
                        ui.add_space(12.0);

                        ui.label(
                            egui::RichText::new(
                                "Kotobase 0.1.0",
                            )
                            .weak(),
                        );

                        ui.label(
                            egui::RichText::new(
                                self.database_path
                                    .display()
                                    .to_string(),
                            )
                            .weak()
                            .small(),
                        );
                    },
                );
            });

        egui::CentralPanel::default().show(
            context,
            |ui| {
                ui.add_space(32.0);

                match self.selected_section {
                    Section::Home => {
                        self.home(ui);
                    }

                    Section::Vocabulary => {
                        self.placeholder(
                            ui,
                            "Vocabulary",
                        );
                    }

                    Section::Kanji => {
                        self.placeholder(
                            ui,
                            "Kanji",
                        );
                    }

                    Section::Grammar => {
                        self.placeholder(
                            ui,
                            "Grammar",
                        );
                    }

                    Section::References => {
                        self.placeholder(
                            ui,
                            "References",
                        );
                    }
                }
            },
        );
    }
}

fn project_path(relative_path: &str) -> PathBuf {
    std::env::current_dir()
        .expect("failed to determine current directory")
        .join(relative_path)
}

fn load_texture(
    context: &egui::Context,
    name: &str,
    path: &Path,
) -> egui::TextureHandle {
    let image = image::open(path)
        .unwrap_or_else(|error| {
            panic!(
                "failed to load logo '{}': {error}",
                path.display()
            )
        })
        .to_rgba8();

    let size = [
        image.width() as usize,
        image.height() as usize,
    ];

    let pixels = image.into_raw();

    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        size,
        &pixels,
    );

    context.load_texture(
        name,
        color_image,
        egui::TextureOptions::LINEAR,
    )
}

fn load_window_icon(
    path: &Path,
) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let image = image::open(path)?.to_rgba8();

    let width = image.width();
    let height = image.height();
    let rgba = image.into_raw();

    Ok(egui::IconData {
        rgba,
        width,
        height,
    })
}

fn database_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_directory = std::env::current_dir()?;

    let data_directory =
        current_directory.join("data");

    std::fs::create_dir_all(&data_directory)?;

    let database_path =
        data_directory.join("kotobase.db");

    ensure_database_directory_exists(
        &database_path,
    )?;

    Ok(database_path)
}

fn ensure_database_directory_exists(
    database_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = database_path
        .parent()
        .ok_or("database path has no parent directory")?;

    std::fs::create_dir_all(parent)?;

    Ok(())
}

async fn verify_database(
    pool: &SqlitePool,
) -> Result<(), sqlx::Error> {
    let vocabulary_count: i64 =
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM vocabulary",
        )
        .fetch_one(pool)
        .await?;

    let kanji_count: i64 =
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM kanji",
        )
        .fetch_one(pool)
        .await?;

    let relationship_count: i64 =
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM relationships",
        )
        .fetch_one(pool)
        .await?;

    let reference_count: i64 =
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM entity_references",
        )
        .fetch_one(pool)
        .await?;

    println!(
        "Stored entities: vocabulary={vocabulary_count}, \
         kanji={kanji_count}, \
         relationships={relationship_count}, \
         references={reference_count}"
    );

    Ok(())
}