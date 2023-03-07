use battle_core::{
    config::ServerConfig,
    map::Map,
    types::{ScenePoint, WorldPoint},
};
use ggez::{
    graphics::{Color, DrawMode, DrawParam, Image, InstanceArray, MeshBuilder, Rect},
    Context, GameResult,
};

pub fn get_map_background_batch(ctx: &mut Context, map: &Map) -> GameResult<InstanceArray> {
    let map_background_image =
        Image::from_path(ctx, map.background_image_path().display().to_string())?;
    let mut map_background_batch = InstanceArray::new(ctx, map_background_image);
    // This batch will never change, add draw param once
    map_background_batch.push(
        DrawParam::new()
            .src(Rect::new(0.0, 0.0, 1.0, 1.0))
            .dest(ScenePoint::new(0.0, 0.0).to_vec2()),
    );

    Ok(map_background_batch)
}

pub fn get_map_interiors_batch(ctx: &mut Context, map: &Map) -> GameResult<InstanceArray> {
    let map_interiors_image =
        Image::from_path(ctx, map.interiors_image_path().display().to_string())?;
    let mut map_interiors_batch = InstanceArray::new(ctx, map_interiors_image);
    // This batch will never change, add draw param once
    map_interiors_batch.push(
        DrawParam::new()
            .src(Rect::new(0.0, 0.0, 1.0, 1.0))
            .dest(ScenePoint::new(0.0, 0.0).to_vec2()),
    );

    Ok(map_interiors_batch)
}

pub fn get_map_decor_batch(ctx: &mut Context, map: &Map) -> GameResult<Vec<InstanceArray>> {
    let mut map_decor_batches = vec![];
    for decor_image_path in map.decor().image_paths() {
        let decor_image = Image::from_path(ctx, decor_image_path.display().to_string())?;
        let batch = InstanceArray::new(ctx, decor_image);
        map_decor_batches.push(batch);
    }

    for decor_batch in map_decor_batches.iter_mut() {
        // FIXME : Why this clear ? Probably nor useful
        decor_batch.clear();
    }

    for tile in map.decor().tiles() {
        let decor_batch = map_decor_batches
            .get_mut(tile.tileset_i)
            .expect("Batch must be here");

        // Tiled draw from bottom left but we draw from top left, so compute a decal
        let dest_decal = tile.tile_height as f32 - map.tile_height() as f32;
        let src_x = tile.tile_x as f32 * tile.relative_tile_width;
        let src_y = tile.tile_y as f32 * tile.relative_tile_height;
        // Destination computation refer to terrain grid (map.terrain.tileset)
        let dest_x = tile.x as f32 * map.tile_width() as f32;
        let dest_y = (tile.y as f32 * map.tile_height() as f32) - dest_decal;

        decor_batch.push(
            DrawParam::new()
                .src(Rect::new(
                    src_x,
                    src_y,
                    tile.relative_tile_width,
                    tile.relative_tile_height,
                ))
                .dest(ScenePoint::new(dest_x, dest_y).to_vec2()),
        );
    }

    Ok(map_decor_batches)
}

pub fn create_debug_terrain_batch(ctx: &mut Context, map: &Map) -> GameResult<InstanceArray> {
    let mut batch = InstanceArray::new(
        ctx,
        Image::from_path(ctx, map.terrain_image_path().display().to_string())?,
    );

    for tile in map.terrain_tiles() {
        let src_x = tile.tile_x as f32 * tile.relative_tile_width;
        let src_y = tile.tile_y as f32 * tile.relative_tile_height;
        let dest_x = tile.x as f32 * tile.tile_width as f32;
        let dest_y = tile.y as f32 * tile.tile_height as f32;
        batch.push(
            DrawParam::new()
                .src(Rect::new(
                    src_x,
                    src_y,
                    tile.relative_tile_width,
                    tile.relative_tile_height,
                ))
                .dest(WorldPoint::new(dest_x, dest_y).to_vec2()),
        );
    }

    Ok(batch)
}

pub fn create_debug_terrain_opacity_mesh_builder(
    map: &Map,
    config: &ServerConfig,
) -> GameResult<MeshBuilder> {
    let mut debug_terrain_opacity_mesh = MeshBuilder::new();
    for tile in map.terrain_tiles() {
        let dest_x = tile.x as f32 * tile.tile_width as f32;
        let dest_y = tile.y as f32 * tile.tile_height as f32;
        let color_modifier = config.terrain_tile_opacity(&tile.type_);
        debug_terrain_opacity_mesh.rectangle(
            DrawMode::fill(),
            Rect::new(
                dest_x,
                dest_y,
                tile.tile_width as f32,
                tile.tile_height as f32,
            ),
            Color {
                r: 0.4 - color_modifier,
                g: 0.4 - color_modifier,
                b: 0.4 - color_modifier,
                a: 1.0,
            },
        )?;
    }
    Ok(debug_terrain_opacity_mesh)
}