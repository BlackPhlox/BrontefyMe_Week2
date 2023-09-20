use benimator::FrameRate;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};

#[derive(Component)]
struct Movement {
    location: Vec3,
    velocity: Vec3,
    is_left: bool,
    speed_scale: f32,
}

// Create the player component
#[derive(Default, Component, Deref, DerefMut)]
struct AnimationState(benimator::State);

#[derive(Component)]
struct Wall {}

#[derive(Component, PartialEq, Eq)]
enum Collider {
    Solid,
    Push,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, initial_setup)
        .add_systems(Update, input_handling)
        .add_systems(Update, movement_system)
        .add_systems(Update, box_collision_system)
        .add_systems(Update, animate_sprite_system)
        .add_systems(Update, brick_collision_system)
        .run();
}
/*
Important system ordering:

1. box_collision_system
2. movement_system
3. input_handling

*/

#[derive(Component)]
struct Animation {
    idle: benimator::Animation,
    run: benimator::Animation,
}

fn initial_setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());

    let idle_anim = Animation {
        idle: benimator::Animation::from_indices(0..=3, FrameRate::from_fps(6.)),
        run: benimator::Animation::from_indices(6..=9, FrameRate::from_fps(6.)),
    };

    let texture_handle = server.load("ryan.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 6, 5, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(2.5)),
            ..Default::default()
        })
        .insert(Movement {
            location: Vec3::from_slice(&[-50., 0., 0.]),
            velocity: Vec3::ZERO,
            is_left: false,
            speed_scale: 125.0,
        })
        .insert(idle_anim)
        .insert(AnimationState::default());

    let box_handle = server.load("crate.png");
    commands
        .spawn(SpriteBundle {
            texture: box_handle,
            ..Default::default()
        })
        .insert(Collider::Push)
        .insert(Movement {
            velocity: Vec3::new(0.0, 0.0, 0.0),
            location: Vec3::new(0.0, 0.0, 0.0),
            is_left: false,
            speed_scale: 70.0,
        });

    for n in 1..6 {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(1300.0 * 0.05, 1300.0 * 0.05)),
                    ..Default::default()
                },
                texture: server.load("wall.png"),
                transform: Transform {
                    translation: Vec3::new(-200.0, -n as f32 * 65.0, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Collider::Solid)
            .insert(Wall {});
    }
}

fn input_handling(
    keys: Res<Input<KeyCode>>,
    mut move_q: Query<&mut Movement, With<TextureAtlasSprite>>,
) {
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

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(
        &mut Movement,
        &mut AnimationState,
        &mut TextureAtlasSprite,
        &Animation,
    )>,
) {
    for (mut movement, mut player, mut texture, animation) in query.iter_mut() {
        // Update the state

        if movement.velocity.x < -0.1 {
            movement.is_left = true;
        } else if movement.velocity.x > 0.1 {
            movement.is_left = false;
        }

        texture.flip_x = movement.is_left;

        if movement.velocity.eq(&Vec3::ZERO) {
            player.update(&animation.idle, time.delta());
        } else {
            player.update(&animation.run, time.delta());
        }

        // Update the texture atlas
        texture.index = player.frame_index();
    }
}

fn movement_system(mut moveable_q: Query<(&mut Movement, &mut Transform)>, time: Res<Time>) {
    for (mut movement, mut transform) in moveable_q.iter_mut() {
        if movement.velocity != Vec3::ZERO {
            let velocity = movement.velocity.normalize();
            let speed_scale = movement.speed_scale;
            movement.location += velocity * speed_scale * time.delta_seconds();
        }
        transform.translation = movement.location;
    }
}

fn box_collision_system(
    mut player_q: Query<(&Transform, &TextureAtlasSprite, &mut Movement), Without<Collider>>,
    mut collider_q: Query<(&mut Movement, &Transform, &Sprite, &Collider), With<Movement>>,
) {
    let (player_transform, player_sprite, mut player_movement) = player_q.single_mut();
    let player_size = player_sprite.custom_size.unwrap_or(Vec2::new(41.6, 51.2));

    for (mut movement, transform, sprite, collider) in collider_q.iter_mut() {
        let collision = collide(
            player_transform.translation,
            player_size,
            transform.translation,
            sprite.custom_size.unwrap_or(Vec2::new(41.0, 42.0)),
        );
        if let Collider::Push = collider {
            if let Some(collision) = collision {
                match collision {
                    Collision::Left => {
                        if player_movement.velocity.x > 0.0 {
                            player_movement.speed_scale = 70.0;
                            movement.velocity.x = 1.0;
                        } else {
                            player_movement.speed_scale = 155.0;
                            movement.velocity.x = 0.0;
                        }
                    }
                    Collision::Right => {
                        if player_movement.velocity.x < 0.0 {
                            player_movement.speed_scale = 70.0;
                            movement.velocity.x = -1.0;
                        } else {
                            player_movement.speed_scale = 155.0;
                            movement.velocity.x = 0.0;
                        }
                    }
                    Collision::Top => {
                        if player_movement.velocity.y < 0.0 {
                            player_movement.speed_scale = 70.0;
                            movement.velocity.y = -1.0
                        } else {
                            player_movement.speed_scale = 155.0;
                            movement.velocity.y = 0.0;
                        }
                    }
                    Collision::Bottom => {
                        if player_movement.velocity.y > 0.0 {
                            player_movement.speed_scale = 70.0;
                            movement.velocity.y = 1.0;
                        } else {
                            player_movement.speed_scale = 155.0;
                            movement.velocity.y = 0.0;
                        }
                    }
                    Collision::Inside => (),
                };
            } else {
                player_movement.speed_scale = 155.0;
                movement.velocity = Vec3::ZERO;
            }
        }
    }
}

fn brick_collision_system(
    mut moveable_q: Query<(&Transform, &Sprite, &mut Movement), With<Movement>>,
    brick_q: Query<(&Transform, &Sprite), With<Wall>>,
) {
    for (transform, sprite, mut movement) in moveable_q.iter_mut() {
        let size = match sprite.custom_size {
            Some(dimension) => dimension,
            None => return,
        };

        for (brick_transform, brick_sprite) in brick_q.iter() {
            let collision = collide(
                transform.translation,
                size,
                brick_transform.translation,
                brick_sprite
                    .custom_size
                    .unwrap_or(Vec2::new(1300.0 * 0.05, 1300.0 * 0.05)),
            );

            if let Some(collision) = collision {
                match collision {
                    Collision::Left => {
                        if movement.velocity.x > 0.0 {
                            movement.velocity.x = 0.0;
                        } else {
                        }
                    }
                    Collision::Right => {
                        if movement.velocity.x < 0.0 {
                            movement.velocity.x = 0.0;
                        } else {
                        }
                    }
                    Collision::Top => {
                        if movement.velocity.y < 0.0 {
                            movement.velocity.y = 0.0;
                        } else {
                        }
                    }
                    Collision::Bottom => {
                        if movement.velocity.y > 0.0 {
                            movement.velocity.y = 0.0;
                        } else {
                        }
                    }
                    Collision::Inside => (),
                }
            }
        }
    }
}
