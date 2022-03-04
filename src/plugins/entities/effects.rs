use bevy::prelude::*;

#[derive(Component)]
struct FlashEffect {
    lifetime: f32,
    current_lifetime: f32,
}

impl Default for FlashEffect {
    fn default() -> Self {
        Self {
            current_lifetime: 0.2,
            lifetime: 0.2,
        }
    }
}

pub struct FlashPlugin;

pub fn spawn_flash(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    transform: &Transform,
    sprite: &str,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(sprite),
            transform: transform.clone(),
            ..Default::default()
        })
        .insert(FlashEffect::default());
}

fn flash_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FlashEffect, &mut Transform)>,
) {
    for (entity, mut flash, mut transform) in query.iter_mut() {
        transform.scale.x = flash.lifetime - flash.current_lifetime;
        transform.scale.y = flash.lifetime - flash.current_lifetime;
        if flash.current_lifetime > 0.0 {
            flash.current_lifetime -= time.delta_seconds();
        } else {
            commands.entity(entity).despawn();
        }
    }
}

impl Plugin for FlashPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(flash_system);
    }
}
