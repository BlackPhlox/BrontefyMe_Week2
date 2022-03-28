use bevy::prelude::*;

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
        .run();
}

fn initial_setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let lonk_handle = server.load("lonk.png");

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(416.0,512.0)/ 10.0),
                ..Default::default()
            },
            texture: lonk_handle,
            ..Default::default()
        })
        .insert(Movement {
            location: Vec3::ZERO,
            velocity: Vec3::ZERO,
        });

    let box_handle = server.load("mbox.png");
    commands.spawn_bundle(SpriteBundle {
        texture: box_handle,
        ..Default::default()
    });
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
        let speed_scale = 105.0;
        movement.location += velocity * speed_scale * time.delta_seconds();
    }
    transform.translation = movement.location;
}

fn box_collision_system(mut commands: Commands, player_q: Query<(&Transform, &Sprite), With<Movement>>, collider_q: Query<(Entity, &Collider, &Transform, &Sprite)>) {
    
}
