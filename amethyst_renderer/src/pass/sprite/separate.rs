//! Simple flat forward drawing pass.

use amethyst_assets::AssetStorage;
use amethyst_core::specs::prelude::{Entities, Join, Read, ReadStorage};
use amethyst_core::transform::GlobalTransform;
use gfx_core::state::{Blend, ColorMask};
use glsl_layout::Uniform;

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use mtl::MaterialTextureSet;
use pass::util::{draw_sprite, get_camera, setup_textures, SpriteArgs, VertexArgs};
use pipe::pass::{Pass, PassData};
use pipe::{DepthMode, Effect, NewEffect};
use sprite::{SpriteRenderInfo, SpriteSheet};
use tex::Texture;
use types::{Encoder, Factory};
use visibility::Visibility;

/// Draw mesh without lighting
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "Self: Pass"))]
pub struct DrawSpriteSeparate {
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
}

impl DrawSpriteSeparate
where
    Self: Pass,
{
    /// Create instance of `DrawSpriteSeparate` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        mask: ColorMask,
        blend: Blend,
        depth: Option<DepthMode>,
    ) -> Self {
        self.transparency = Some((mask, blend, depth));
        self
    }
}

impl<'a> PassData<'a> for DrawSpriteSeparate {
    type Data = (
        Entities<'a>,
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Read<'a, AssetStorage<SpriteSheet>>,
        Read<'a, AssetStorage<Texture>>,
        Read<'a, MaterialTextureSet>,
        Option<Read<'a, Visibility>>,
        ReadStorage<'a, SpriteRenderInfo>,
        ReadStorage<'a, GlobalTransform>,
    );
}

impl Pass for DrawSpriteSeparate {
    fn compile(&mut self, effect: NewEffect) -> Result<Effect> {
        use std::mem;
        let mut builder = effect.simple(VERT_SRC, FRAG_SRC);
        builder
            .with_raw_constant_buffer(
                "VertexArgs",
                mem::size_of::<<VertexArgs as Uniform>::Std140>(),
                1,
            )
            .with_raw_constant_buffer(
                "SpriteArgs",
                mem::size_of::<<SpriteArgs as Uniform>::Std140>(),
                1,
            );
        setup_textures(&mut builder, &TEXTURES);
        match self.transparency {
            Some((mask, blend, depth)) => builder.with_blended_output("color", mask, blend, depth),
            None => builder.with_output("color", Some(DepthMode::LessEqualWrite)),
        };
        builder.build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        _factory: Factory,
        (
            entities,
            active,
            camera,
            sprite_sheet_storage,
            tex_storage,
            material_texture_set,
            visibility,
            sprite_render_info,
            global,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        match visibility {
            None => for (_entity, sprite_render_info, global) in
                (&*entities, &sprite_render_info, &global).join()
            {
                draw_sprite(
                    encoder,
                    effect,
                    sprite_render_info,
                    &sprite_sheet_storage,
                    &tex_storage,
                    &material_texture_set,
                    camera,
                    Some(global),
                );
            },
            Some(ref visibility) => {
                for (_entity, sprite_render_info, global, _) in (
                    &*entities,
                    &sprite_render_info,
                    &global,
                    &visibility.visible_unordered,
                ).join()
                {
                    draw_sprite(
                        encoder,
                        effect,
                        sprite_render_info,
                        &sprite_sheet_storage,
                        &tex_storage,
                        &material_texture_set,
                        camera,
                        Some(global),
                    );
                }

                for entity in &visibility.visible_ordered {
                    if let Some(sprite_render_info) = sprite_render_info.get(*entity) {
                        draw_sprite(
                            encoder,
                            effect,
                            sprite_render_info,
                            &sprite_sheet_storage,
                            &tex_storage,
                            &material_texture_set,
                            camera,
                            global.get(*entity),
                        );
                    }
                }
            }
        }
    }
}
