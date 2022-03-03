use bevy::prelude::*;

pub struct BoardPlugin {
    pub rings: i32,
}

fn hello_world<const RINGS: i32>(mut commands: Commands, asset_server: Res<AssetServer>) {
    // commands.spawn_bundle(SpriteBundle {
    //     texture: asset_server.load("tile.png"),
    //     ..Default::default()
    // });
    let rows = if RINGS % 2 == 0 { RINGS + 1 } else { RINGS };
    for ring in 0..rows {
        let hex_count = (RINGS) - (ring - (RINGS) / 2).abs();
        for i in 0..hex_count {
            let x = (i as f32 - ((hex_count - 1) as f32 / 2.0)) * 32.0;
            let y = (ring - (RINGS / 2)) as f32 * 21.0;
            let transform = Transform::from_translation(Vec3::new(x, y, 100.0 - (y * 0.1)));

            commands.spawn_bundle(SpriteBundle {
                texture: asset_server.load("tile.png"),
                transform: transform,
                ..Default::default()
            });
        }
    }
}

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(hello_world::<5>);
    }
}
