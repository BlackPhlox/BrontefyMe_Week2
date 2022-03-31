use std::{time::Duration, ops::Deref};
use benimator::{Play, SpriteSheetAnimation, AnimationPlugin};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};

#[derive(Component)]
struct Movement {
    location: Vec3,
    velocity: Vec3,
    is_left: bool,
}

#[derive(Component)]
enum Collider {
    Solid,
}

fn main() {
    App::new()
        .init_resource::<Animations>()
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin::default())
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_animations)
        .add_startup_system(initial_setup)
        .add_system(input_handling)
        .add_system(movement_system)
        .add_system(box_collision_system)
        .add_system(animate_sprite_system)
        .run();
}

#[derive(Default)]
struct Animations {
    idle: Handle<SpriteSheetAnimation>,
    moving: Handle<SpriteSheetAnimation>,
}

fn setup_animations (
    mut handles: ResMut<Animations>,
    mut assets: ResMut<Assets<SpriteSheetAnimation>>,
) {
    handles.idle = assets.add(SpriteSheetAnimation::from_range(
        0..=3,
        Duration::from_millis(150),
    ));
    handles.moving = assets.add(SpriteSheetAnimation::from_range(
        6..=9,
        Duration::from_millis(150),
    ));
}

fn initial_setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    anim: Res<Animations>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = server.load("hero.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 6, 5);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(2.5)),
            ..Default::default()
        })
        .insert(Movement {
            location: Vec3::from_slice(&[-50.,0.,0.]),
            velocity: Vec3::ZERO,
            is_left: false,
        })
        .insert(anim.idle.clone())
        .insert(Play);

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

fn animate_sprite_system(
    animations: Res<Animations>,
    mut move_q: Query<&mut Movement>,
    mut query: Query<&mut Handle<SpriteSheetAnimation>>,
    mut sprite_q: Query<&mut TextureAtlasSprite>,
){
    let mut movement = move_q.single_mut();
    let mut animation = query.single_mut();

    if movement.velocity.x < -0.1 {
        movement.is_left = true;
    } else if movement.velocity.x > 0.1 {
        movement.is_left = false;
    }

    let sprite_atlas = sprite_q.get_single_mut();
    match sprite_atlas {
        Ok(mut x) => x.flip_x = movement.is_left,
        Err(_) => println!("Oh no! Couldn't find hero sprite"),
    }
    
    if movement.velocity.eq(&Vec3::ZERO){
        *animation = animations.idle.clone();
    } else {
        *animation = animations.moving.clone();
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
    mut player_q: Query<(&Transform, &TextureAtlasSprite, &mut Movement)>,
    collider_q: Query<(&Transform, &Sprite), With<Collider>>,
) {
    let (player_transform, player_sprite, mut player_movement) = player_q.single_mut();
    let player_size = player_sprite.custom_size.unwrap_or(Vec2::new(41.6, 51.2));

    for (transform, sprite) in collider_q.iter() {
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
