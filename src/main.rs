use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};

#[derive(Component)]
struct Movement {
    location: Vec3,
    velocity: Vec3,
}

#[derive(Component)]
enum Collider {
    Solid,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(initial_setup)
        .add_system(input_handling)
        .add_system(movement_system)
        .add_system(box_collision_system)
        .run();
}

fn initial_setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let lonk_handle = server.load("lonk.png");

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(416.0, 512.0) / 10.0),
                ..Default::default()
            },
           global_transform: GlobalTransform {
                translation: Vec3::new(100.0,0.0, 0.0),
                ..Default::default()
            },
            texture: lonk_handle,
            ..Default::default()
        })
        .insert(Movement {
            location: Vec3::new(100.0, 0.0, 0.0),
            velocity: Vec3::ZERO,
        });

    let box_handle = server.load("mbox.png");
    commands
        .spawn_bundle(SpriteBundle {
            texture: box_handle,
            ..Default::default()
        })
        .insert(Collider::Solid);
}

fn input_handling(keys: Res<Input<KeyCode>>, mut move_q: Query<&mut Movement>) {
    let mut movement = move_q.single_mut();

    movement.velocity = Vec3::ZERO;

    for key in keys.get_pressed() {
        movement.velocity += match key {
            KeyCode::W => Vec3::new(0.0, 1.0, 0.0),
            KeyCode::A => Vec3::new(-1.0, 0.0, 0.0),
            KeyCode::S => Vec3::new(0.0, -1.0, 0.0),
            KeyCode::D => Vec3::new(1.0, 0.0, 0.0),
            _ => Vec3::ZERO,
        }
    }
}

fn movement_system(mut player_q: Query<(&mut Movement, &mut Transform)>, time: Res<Time>) {
    let (mut movement, mut transform) = player_q.single_mut();
    

    if movement.velocity != Vec3::ZERO {
        let velocity = movement.velocity.normalize();
        let speed_scale = 125.0;
        movement.location += velocity * speed_scale * time.delta_seconds();
    }
    transform.translation = movement.location;
}

fn box_collision_system(
    mut commands: Commands,
    mut player_q: Query<(&Transform, &Sprite, &mut Movement)>,
    collider_q: Query<(Entity, &Collider, &Transform, &Sprite)>,
) {
    let (player_transform, player_sprite, mut player_movement) = player_q.single_mut();
    let player_size = player_sprite.custom_size.unwrap_or(Vec2::new(41.6, 51.2));

    for (collider_entity, collider, transform, sprite) in collider_q.iter() {
        let collision = collide(
            player_transform.translation,
            player_size,
            transform.translation,
            sprite.custom_size.unwrap_or(Vec2::new(41.0,42.0)),
        );
        if let Some(collision) = collision {
            match collision {
                Collision::Left  => {if player_movement.velocity.x > 0.0 {
                    player_movement.velocity.x = 0.0;
                }},
                Collision::Right => {if player_movement.velocity.x < 0.0 {
                    player_movement.velocity.x = 0.0;
                }},
                Collision::Top => { if player_movement.velocity.y < 0.0 {
                    player_movement.velocity.y = 0.0;
                }},
                Collision::Bottom => { if player_movement.velocity.y > 0.0 {
                    player_movement.velocity.y = 0.0; 
                }},
            };
        }
    }
}
