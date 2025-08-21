use eframe::egui::Color32;
use eframe::wgpu;

use super::Texture;
use crate::util::UiExt;
use eframe::egui::Sense;
use eframe::egui_wgpu::RenderState;
use eframe::epaint::TextureId;
use eframe::epaint::mutex::RwLock;
use eframe::epaint::vec2;
use either::Either;
use either::Either::Left;
use linked_hash_map::LinkedHashMap;
use poll_promise::Promise;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use std::rc::Rc;
use std::sync::Arc;
use tiger_pkg::TagHash;
pub type LoadedTexture = (Arc<Texture>, TextureId);

pub(crate) type TextureCacheMap = LinkedHashMap<
    TagHash,
    Either<Option<LoadedTexture>, Promise<Option<LoadedTexture>>>,
    BuildHasherDefault<FxHasher>,
>;

#[derive(Clone)]
pub struct TextureCache {
    pub render_state: RenderState,
    pub(crate) cache: Rc<RwLock<TextureCacheMap>>,
    pub(crate) loading_placeholder: LoadedTexture,
}

impl TextureCache {
    pub fn new(render_state: RenderState) -> Self {
        let loading_placeholder =
            Texture::load_png(&render_state, include_bytes!("../../../assets/loading.png"))
                .unwrap();

        let loading_placeholder_id = render_state.renderer.write().register_native_texture(
            &render_state.device,
            &loading_placeholder.view,
            wgpu::FilterMode::Linear,
        );

        Self {
            render_state,
            cache: Rc::new(RwLock::new(TextureCacheMap::default())),
            loading_placeholder: (Arc::new(loading_placeholder), loading_placeholder_id),
        }
    }

    pub fn is_loading_textures(&self) -> bool {
        self.cache
            .read()
            .iter()
            .any(|(_, v)| matches!(v, Either::Right(_)))
    }

    pub fn get_or_default(&self, hash: TagHash) -> LoadedTexture {
        self.get_or_load(hash)
            .unwrap_or_else(|| self.loading_placeholder.clone())
    }

    pub fn get_or_load(&self, hash: TagHash) -> Option<LoadedTexture> {
        let mut cache = self.cache.write();

        let c = cache.remove(&hash);

        let texture = if let Some(Either::Left(r)) = c {
            cache.insert(hash, Left(r.clone()));
            r.clone()
        } else if let Some(Either::Right(p)) = c {
            if let std::task::Poll::Ready(r) = p.poll() {
                cache.insert(hash, Left(r.clone()));
                return r.clone();
            } else {
                cache.insert(hash, Either::Right(p));
                None
            }
        } else if c.is_none() {
            cache.insert(
                hash,
                Either::Right(Promise::spawn_async(Self::load_texture_task(
                    self.render_state.clone(),
                    hash,
                ))),
            );

            None
        } else {
            None
        };

        drop(cache);
        self.truncate();

        texture
    }

    pub(crate) async fn load_texture_task(
        render_state: RenderState,
        hash: TagHash,
    ) -> Option<LoadedTexture> {
        let texture = match Texture::load(&render_state, hash) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to load texture {hash}: {e}");
                return None;
            }
        };

        let id = render_state.renderer.write().register_native_texture(
            &render_state.device,
            &texture.view,
            wgpu::FilterMode::Linear,
        );
        Some((Arc::new(texture), id))
    }

    pub(crate) const MAX_TEXTURES: usize = 2048;
    pub(crate) fn truncate(&self) {
        let mut cache = self.cache.write();
        while cache.len() > Self::MAX_TEXTURES {
            if let Some((_, Either::Left(Some((_, tid))))) = cache.pop_front() {
                self.render_state.renderer.write().free_texture(&tid);
            }
        }
    }
}
