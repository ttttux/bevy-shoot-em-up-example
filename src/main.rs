use std::time::{Duration, SystemTime};
use bevy::prelude::*;
const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_systems(Startup, setup)
        .add_systems(Update, execute_animations)
        .add_systems(Update, player_movement_system)
        .add_systems(Update, player_weapons_system)
        .add_systems(Update, player_shoot_system)
        .add_systems(Update, move_enemies)
        .run();
}

fn trigger_animation<S: Component>(mut query: Query<&mut AnimationConfig, With<S>>) {
    let mut animation = query.single_mut();
    animation.frame_timer = AnimationConfig::timer_from_fps(animation.fps);
}

#[derive(Component)]
struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
}

impl AnimationConfig {
    fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

fn execute_animations(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas)>,
) {
    for (mut config, mut atlas) in &mut query {
        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished() {
            atlas.index += 1;

            if atlas.index > config.last_sprite_index {
                atlas.index = config.first_sprite_index;
            }

            config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
        }
    }
}

#[derive(Component)]
struct Enemy {
    is_hit: bool,
    position: Vec3,
    movement_speed: f32
}

#[derive(Component)]
struct Cooldown {
    last_time: u128
}

#[derive(Component)]
struct LaserSprite;

#[derive(Component)]
struct Laser {
    movement_speed: f32,
    laser_sprite: LaserSprite
}

#[derive(Component)]
struct PlayerSprite;

#[derive(Component)]
struct Player {
    movement_speed: f32,
    player_sprite: PlayerSprite,
    position: Vec3
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let enemy_a_handle = asset_server.load("Spaceship-shooter-gamekit/Assets/spritesheets/enemy-medium.png");
    //let enemy_b_handle = asset_server.load("Spaceship-shooter-gamekit/Assets/spritesheets/enemy-small.png");

    let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 16), 2, 1, None, None);
    let texture_atlas_layout_medium = texture_atlas_layouts.add(layout);

    let animation_config_2 = AnimationConfig::new(0, 1, 10);

    commands.spawn(Camera2dBundle::default());

    //let horizontal_margin = BOUNDS.x / 4.0;
    //let vertical_margin = BOUNDS.y / 4.0;

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(3.0))
                .with_translation(Vec3::new(-100.0, 300.0, 0.0)),
            texture: enemy_a_handle.clone(),
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout_medium.clone(),
            index: animation_config_2.first_sprite_index,
        },
        Enemy {
            is_hit: false,
            position: Vec3::new(0.0, 0.0, 0.0),
            movement_speed: 200.0,
        }
    ));

    /*commands.spawn((
        SpriteBundle {
            texture: enemy_a_handle,
            transform: Transform::from_xyz(0.0, 0.0 - vertical_margin, 0.0),
            ..default()
        },
    ));

    commands.spawn((
        SpriteBundle {
            texture: enemy_b_handle.clone(),
            transform: Transform::from_xyz(0.0 + horizontal_margin, 0.0, 0.0),
            ..default()
        },
    ));
    commands.spawn((
        SpriteBundle {
            texture: enemy_b_handle,
            transform: Transform::from_xyz(0.0, 0.0 + vertical_margin, 0.0),
            ..default()
        },
    ));*/

    let texture = asset_server.load("Spaceship-shooter-gamekit/Assets/spritesheets/ship.png");

    let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 24), 5, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let animation_config_1 = AnimationConfig::new(0, 9, 10);

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(3.0))
                .with_translation(Vec3::new(0.0, 0.0, 0.0)),
            texture: texture.clone(),
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: animation_config_1.first_sprite_index,
        },
        PlayerSprite,
        Cooldown{last_time: now_as_u128()},
        animation_config_1,
        Player {
            movement_speed: 500.0,
            player_sprite: PlayerSprite,
            position: Vec3::new(0.0, 0.0, 0.0)
        }
    ));

    commands.spawn(TextBundle {
        text: Text::from_section(
            "Move: Arrow Keys\nShoot: Space",
            TextStyle::default(),
        ),
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        ..default()
    });
}

fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let (mut ship, mut transform) = query.single_mut();

    let mut movement_x = 0.0;
    let mut movement_y = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        movement_x -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        movement_x += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        movement_y += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowDown) {
        movement_y -= 1.0;
    }

    let movement_distance_x = movement_x * ship.movement_speed * time.delta_seconds();
    let movement_distance_y = movement_y * ship.movement_speed * time.delta_seconds();
    transform.translation.y += movement_distance_y;
    transform.translation.x += movement_distance_x;
    ship.position.x = transform.translation.x;
    ship.position.y = transform.translation.y;

    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}


fn player_weapons_system(
    time: Res<Time>,
    mut query: Query<(&Laser, &mut Transform)>
) {
    for (shot, mut transform) in query.iter_mut() {
        let movement_distance_y = 2.0 * shot.movement_speed * time.delta_seconds();
        transform.translation.y += movement_distance_y;

        let extents = Vec3::from((BOUNDS / 2.0, 0.0));
        transform.translation = transform.translation.min(extents).max(-extents);
    }

}
fn player_shoot_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut query: Query<(&mut Cooldown, &Player)>,
    keyboard_input: Res<ButtonInput<KeyCode>>
) {

    for (mut cooldown, ship) in query.iter_mut() {
        let texture = asset_server.load("Spaceship-shooter-gamekit/Assets/spritesheets/laser-bolts.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 2, 2, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let animation_config_1 = AnimationConfig::new(2, 3, 10);
        if keyboard_input.pressed(KeyCode::Space) {
            if cooldown.last_time + 250 <= now_as_u128() {
                cooldown.last_time = now_as_u128();
                commands.spawn((
                    SpriteBundle {
                        transform: Transform::from_scale(Vec3::splat(3.0))
                            .with_translation(Vec3::new(ship.position.x, ship.position.y + 6.0, 0.0)),
                        texture,
                        ..default()
                    },
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: animation_config_1.first_sprite_index,
                    },
                    LaserSprite,
                    animation_config_1,
                    Laser {
                        movement_speed: 500.0,
                        laser_sprite: LaserSprite,
                    }
                ));
            }
        }
    }
}

fn now_as_u128() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
}

fn move_enemies(
    time: Res<Time>,
    mut query: Query<(&mut Enemy, &mut Transform)>,
    ship_query: Query<(&Player)>
) {
    let (mut enemy, mut transform) = query.single_mut();
    let (ship) = ship_query.single();

    let mut movement_x = 0.0;
    let mut movement_y = 0.0;

    if ship.position.x < enemy.position.x {
        movement_x -= 1.0;
    }

    if ship.position.x > enemy.position.x {
        movement_x += 1.0;
    }

    if ship.position.y > enemy.position.y{
        movement_y += 1.0;
    }

    if ship.position.y < enemy.position.y {
        movement_y -= 1.0;
    }

    let movement_distance_x = movement_x * enemy.movement_speed * time.delta_seconds();
    let movement_distance_y = movement_y * enemy.movement_speed * time.delta_seconds();
    transform.translation.y += movement_distance_y;
    transform.translation.x += movement_distance_x;
    enemy.position.x = transform.translation.x;
    enemy.position.y = transform.translation.y;

    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}