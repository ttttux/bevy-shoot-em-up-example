    #![feature(exact_size_is_empty)]
    #![feature(random)]
    #![feature(dec2flt)]
    extern crate core;

    use std::random::random;
    use std::time::{Duration, SystemTime};
    use bevy::math::NormedVectorSpace;
    use bevy::prelude::*;
    use rand::Rng;
    use bevy_framepace::*;

    const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

    fn main() {
        App::new()
            .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
            .add_plugins(bevy_framepace::FramepacePlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, execute_animations)
            .add_systems(Update, player_movement_system)
            .add_systems(Update, player_weapons_system)
            .add_systems(Update, player_shoot_system)
            .add_systems(Update, enemy_movement_system)
            .add_systems(Update, enemy_kill_system)
            .add_systems(Update, explosion_and_laser_termination_system)
            .add_systems(Update, player_kill_system)
            .add_systems(Update, enemy_spawn_system)
            .add_systems(Update, spawn_timer_system)
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
    struct ScoreCounter {
        score: f32,
    }

    #[derive(Component)]
    struct ScoreCounterText;

    #[derive(Component)]
    struct SpawnTimer {
        timer: f32
    }

    #[derive(Component)]
    struct Explosion {
        frame_timer: f32
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
        laser_sprite: LaserSprite,
        position: Vec3
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
        mut settings: ResMut<FramepaceSettings>
    ) {
        settings.limiter = Limiter::from_framerate(60.0);

        commands.spawn(Camera2dBundle::default());

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

        commands.spawn(
            SpawnTimer {
                timer: 30.0
            }
        );

        commands.spawn(
            ScoreCounter {
                score: 0.0
            }
        );

        commands.spawn((TextBundle {
            text: Text::from_section(
                "Score: 000",
                TextStyle::default(),
            ),
            style: Style {
                position_type: PositionType::Absolute,
            top: Val::Px(640.0),
                left: Val::Px(1100.0),
                ..default()
            },
            ..default()
        },ScoreCounter {
            score: 0.0,
        }));
    }

    fn player_movement_system(
        time: Res<Time>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut query: Query<(&mut Player, &mut Transform)>,
    ) {
        if query.iter().is_empty() {
            return;
        }

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

    fn enemy_kill_system(
        mut commands: Commands,
        mut enemy_query: Query<(Entity, &Transform), With<Enemy>>,
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        mut shot_query: Query<(Entity, &Transform), With<Laser>>,
        mut text_query: Query<(&mut Text, &mut ScoreCounter)>,
    ) {
        if shot_query.iter().is_empty() || enemy_query.iter().is_empty() {
            return;
        }


        for (enemy_entity, enemy_transform) in &enemy_query {
            for (shot, shot_transform) in &shot_query {
                if enemy_transform.translation.y.distance(shot_transform.translation.y) < 15.5 && enemy_transform.translation.x.distance(shot_transform.translation.x) < 45.5 {
                    let (mut text_bundle,mut score_counter) = text_query.single_mut();

                    score_counter.score += 1.0;
                    let text = format!("Score: {}", score_counter.score);
                    text_bundle.sections = Text::from_section(text, Default::default()).sections;

                    commands.entity(enemy_entity).despawn();
                    commands.entity(shot).despawn();
                    let texture = asset_server.load("Spaceship-shooter-gamekit/Assets/spritesheets/explosion.png");

                    let layout = TextureAtlasLayout::from_grid(UVec2::new(80 / 5, 16), 5, 1, None, None);
                    let texture_atlas_layout = texture_atlas_layouts.add(layout);

                    let animation_config_1 = AnimationConfig::new(0, 4, 10);

                    commands.spawn((
                        SpriteBundle {
                            transform: Transform::from_scale(Vec3::splat(6.0))
                                .with_translation(Vec3::new(enemy_transform.translation.x, enemy_transform.translation.y, 0.0)),
                            texture,
                            ..default()
                        },
                        TextureAtlas {
                            layout: texture_atlas_layout,
                            index: animation_config_1.first_sprite_index,
                        },
                        animation_config_1,
                        Explosion {
                            frame_timer: 0.0
                        }
                    ));
                }
            }
        }
    }

    fn player_weapons_system(
        time: Res<Time>,
        mut query: Query<(&mut Laser, &mut Transform)>
    ) {
        for (mut shot, mut transform) in query.iter_mut() {
            let movement_distance_y = 2.0 * shot.movement_speed * time.delta_seconds();
            transform.translation.y += movement_distance_y;
            shot.position.y = transform.translation.y;

            let extents = Vec3::from((BOUNDS, 0.0));
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
                            position: Vec3::new(ship.position.x, ship.position.y + 6.0, 0.0)
                        }
                    ));
                }
            }
        }
    }

    fn now_as_u128() -> u128 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
    }

    fn enemy_movement_system(
        time: Res<Time>,
        mut query: Query<(&mut Enemy, &mut Transform)>,
        ship_query: Query<(&Player)>
    ) {
        if query.is_empty() || ship_query.is_empty(){
            return;
        }

        for (mut enemy, mut transform) in &mut query{
            let (ship) = ship_query.single();

            let mut movement_x = 0.0;
            let mut movement_y = 0.0;

            if ship.position.x < enemy.position.x {
                movement_x -= 1.0;
            }

            if ship.position.x > enemy.position.x {
                movement_x += 1.0;
            }

            if ship.position.y > enemy.position.y {
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

            let extents = 640.0;
            transform.translation.y = transform.translation.y.min(extents).max(-extents);
        }
    }

    fn explosion_and_laser_termination_system(
        mut commands: Commands,
        mut query: Query<(Entity, &mut Explosion), With<Explosion>>,
        mut shot_query: Query<(Entity, &mut Laser), With<Laser>>
    ) {
        if !query.iter().is_empty() {
            for (entity, mut explosion) in &mut query {
                explosion.frame_timer += 1.0;

                if explosion.frame_timer > 30.0 {
                    commands.entity(entity).despawn();
                }
            }
        }
        if !shot_query.iter().is_empty() {
            for (laser_entity, shot) in &mut shot_query {
                if shot.position.y >= BOUNDS.y/2.0  {
                    commands.entity(laser_entity).despawn();
                }
            }
        }
    }


    fn player_kill_system(
        mut commands: Commands,
        mut query: Query<(Entity, &Transform), With<Player>>,
        enemy_query: Query<&Transform, With<Enemy>>,
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>
    ) {
        if !query.iter().is_empty() && !enemy_query.iter().is_empty() {
            let (entity, player) = query.single_mut();
            for enemy in &enemy_query {
                if player.translation.y.distance(enemy.translation.y) < 15.5 && player.translation.x.distance(enemy.translation.x) < 45.5  {

                    commands.entity(entity).despawn();
                    let texture = asset_server.load("Spaceship-shooter-gamekit/Assets/spritesheets/explosion.png");

                    let layout = TextureAtlasLayout::from_grid(UVec2::new(80 / 5, 16), 5, 1, None, None);
                    let texture_atlas_layout = texture_atlas_layouts.add(layout);

                    let animation_config_1 = AnimationConfig::new(0, 4, 10);

                    commands.spawn((
                        SpriteBundle {
                            transform: Transform::from_scale(Vec3::splat(6.0))
                                .with_translation(Vec3::new(player.translation.x, player.translation.y, 0.0)),
                            texture,
                            ..default()
                        },
                        TextureAtlas {
                            layout: texture_atlas_layout,
                            index: animation_config_1.first_sprite_index,
                        },
                        animation_config_1,
                        Explosion {
                            frame_timer: 0.0
                        }
                    ));
                }
            }
        }
    }

    fn enemy_spawn_system(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        query: Query<&SpawnTimer>
    ) {
        let spawn_timer = query.single();
        if spawn_timer.timer <= 1.0 {
            let enemy_a_handle = asset_server.load("Spaceship-shooter-gamekit/Assets/spritesheets/enemy-medium.png");

            let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 16), 2, 1, None, None);
            let texture_atlas_layout_medium = texture_atlas_layouts.add(layout);

            let animation_config_2 = AnimationConfig::new(0, 1, 10);

            let randgennumb: f32 = rand::thread_rng().gen_range(-1200..1200).to_string().parse().unwrap();

            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_scale(Vec3::splat(3.0))
                        .with_translation(Vec3::new(randgennumb, 640.0, 0.0)),
                    texture: enemy_a_handle.clone(),
                    ..default()
                },
                TextureAtlas {
                    layout: texture_atlas_layout_medium.clone(),
                    index: animation_config_2.first_sprite_index,
                },
                Enemy {
                    is_hit: false,
                    position: Vec3::new(randgennumb, 300.0, 0.0),
                    movement_speed: 200.0,
                }
            ));
        }
    }

    fn spawn_timer_system(
        mut query: Query<&mut SpawnTimer>
    ) {
        let mut spawn_timer = query.single_mut();
        spawn_timer.timer -= 1.0;
        if spawn_timer.timer < 1.0 {
            spawn_timer.timer = 30.0;
        }
    }
