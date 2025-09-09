use std::sync::Arc;

use crate::gui::TachyscopeApp;

use clap::Parser;
use eframe::{
    egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew},
    wgpu::{Backends, InstanceDescriptor},
};
use env_logger::Env;
use game_detector::InstalledGame;
use log::info;
use tiger_pkg::{DestinyVersion, GameVersion, PackageManager};
use tracing_subscriber::layer::SubscriberExt;

mod gui;
mod util;

#[derive(Clone, Debug, clap::Parser)]
pub struct Args {
    pub packages_path: Option<String>,

    #[arg(short, value_enum)]
    pub version: Option<tiger_pkg::GameVersion>,
}

fn main() -> eframe::Result<()> {
    // Limit rayon threads to 4 as to not kill PCs when search is running
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let _rt_guard = rt.enter();

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
    )
    .expect("setup tracy layer");

    env_logger::Builder::from_env(
        Env::default()
            .default_filter_or("info,eframe=warn,wgpu_core=warn,wgpu_hal=error,naga=warn"),
    )
    .init();
    let args = Args::parse();

    let packages_path = if let Some(packages_path) = args.packages_path {
        packages_path
    } else if let Some(path) = find_d2_packages_path() {
        let mut path = std::path::PathBuf::from(path);
        path.push("packages");
        path.to_str().unwrap().to_string()
    } else {
        panic!("Could not find Destiny 2 packages directory");
    };

    info!(
        "Initializing package manager for version {:?} at '{}'",
        args.version, packages_path
    );
    let pm = PackageManager::new(
        packages_path,
        args.version
            .unwrap_or(GameVersion::Destiny(DestinyVersion::Destiny2TheEdgeOfFate)),
        None,
    )
    .unwrap();

    tiger_pkg::initialize_package_manager(&Arc::new(pm));

    let wgpu_setup = WgpuSetup::CreateNew(WgpuSetupCreateNew {
        instance_descriptor: InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        },
        // device_descriptor: Arc::new(|_adapter| wgpu::DeviceDescriptor {
        //     required_features: Features::TEXTURE_FORMAT_16BIT_NORM
        //         | Features::TEXTURE_COMPRESSION_BC
        //         | Features::TEXTURE_BINDING_ARRAY,
        //     ..Default::default()
        // }),
        ..Default::default()
    });

    let wgpu_config = WgpuConfiguration {
        wgpu_setup,
        ..Default::default()
    };

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        persist_window: true,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        wgpu_options: wgpu_config,
        ..Default::default()
    };
    eframe::run_native(
        "Tachyscope",
        native_options,
        Box::new(|cc| Ok(Box::new(TachyscopeApp::new(cc)))),
    )
}

fn find_d2_packages_path() -> Option<String> {
    let mut installations = game_detector::find_all_games();
    installations.retain(|i| match i {
        InstalledGame::Steam(a) => a.appid == 1085660,
        InstalledGame::EpicGames(m) => m.display_name == "Destiny 2",
        InstalledGame::MicrosoftStore(p) => p.app_name == "Destiny2PCbasegame",
        _ => false,
    });

    info!("Found {} Destiny 2 installations", installations.len());

    // Sort installations, weighting Steam > Epic > Microsoft Store
    installations.sort_by_cached_key(|i| match i {
        InstalledGame::Steam(_) => 0,
        InstalledGame::EpicGames(_) => 1,
        InstalledGame::MicrosoftStore(_) => 2,
        _ => 3,
    });

    match installations.first() {
        Some(InstalledGame::Steam(a)) => Some(a.game_path.clone()),
        Some(InstalledGame::EpicGames(m)) => Some(m.install_location.clone()),
        Some(InstalledGame::MicrosoftStore(p)) => Some(p.path.clone()),
        _ => None,
    }
}
