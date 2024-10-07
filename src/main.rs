    #![feature(exact_size_is_empty)]
    #![feature(random)]
    #![feature(dec2flt)]
    extern crate core;

    use std::random::random;
    use std::time::{Duration, SystemTime};
    use bevy::color::palettes::css::{CRIMSON, WHITE};
    use bevy::math::NormedVectorSpace;
    use bevy::prelude::*;
    use rand::Rng;
    use bevy_framepace::*;
    use config::Value;

    const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

    fn main() {
        App::new()
            .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))// prevents blurry sprites
            .add_plugins(bevy_framepace::FramepacePlugin)
            .init_state::<GameState>()
            .init_state::<MenuState>()
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Over), menu_setup)
            .add_systems(OnEnter(GameState::Over), main_menu_setup)
            .add_systems(Update, (menu_action, button_system).run_if(in_state(GameState::Over)))
            .add_systems(OnEnter(GameState::Game), setup)
            .add_systems(OnExit(GameState::Game), clear_after_game_over)
            .add_systems(OnExit(GameState::Over), despawn_screen::<OnMainMenuScreen>)       
            .add_systems(Startup, splash_setup)
            .add_systems(Update, countdown.after(splash_setup))
            .add_systems(Update, execute_animations.after(setup))
            .add_systems(Update, player_movement_system.after(setup))
            .add_systems(Update, player_weapons_system.after(setup))
            .add_systems(Update, player_shoot_system.after(setup))
            .add_systems(Update, enemy_movement_system.after(setup))
            .add_systems(Update, enemy_kill_system.after(setup))
            .add_systems(Update, explosion_and_laser_termination_system.after(setup))
            .add_systems(Update, player_kill_system.after(setup))
            .add_systems(Update, enemy_spawn_system.after(setup))
            .add_systems(Update, spawn_timer_system.after(setup))
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
    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    enum MenuState {
        Main,
        Settings,
        SettingsDisplay,
        SettingsSound,
        #[default]
        Disabled,
    }

    // Tag component used to tag entities added on the main menu screen
    #[derive(Component)]
    struct OnMainMenuScreen;

    // Tag component used to tag entities added on the settings menu screen
    #[derive(Component)]
    struct OnSettingsMenuScreen;

    // Tag component used to tag entities added on the display settings menu screen
    #[derive(Component)]
    struct OnDisplaySettingsMenuScreen;

    // Tag component used to tag entities added on the sound settings menu screen
    #[derive(Component)]
    struct OnSoundSettingsMenuScreen;


    const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    // Tag component used to mark which setting is currently selected
    #[derive(Component)]
    struct SelectedOption;

    // All actions that can be triggered from a button click
    #[derive(Component)]
    enum MenuButtonAction {
        Play,
        Settings,
        SettingsDisplay,
        SettingsSound,
        BackToMainMenu,
        BackToSettings,
        Quit,
    }


    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    enum GameState {
        #[default]
        Splash,
        Menu,
        Game,
        Over
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

    fn setup_camera(
        mut commands: Commands
    ) {
        commands.spawn(Camera2dBundle::default());
    }

    fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        mut settings: ResMut<FramepaceSettings>
    ) {
        settings.limiter = Limiter::from_framerate(60.0);

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
        mut game_state: ResMut<NextState<GameState>>,
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

                    game_state.set(GameState::Over);
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
        if query.is_empty() {
            return;
        }

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
        if query.is_empty() {
            return;
        }

        let mut spawn_timer = query.single_mut();
        spawn_timer.timer -= 1.0;
        if spawn_timer.timer < 1.0 {
            spawn_timer.timer = 30.0;
        }
    }

    #[derive(Component)]
    struct OnSplashScreen;

    // Newtype to use a `Timer` for this screen as a resource
    #[derive(Resource, Deref, DerefMut)]
    struct SplashTimer(Timer);

    fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        let icon = asset_server.load("branding/icon.png");
        // Display the logo
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                },
                OnSplashScreen,
            ))
            .with_children(|parent| {
                parent.spawn(ImageBundle {
                    style: Style {
                        // This will set the logo to be 200px wide, and auto adjust its height
                        width: Val::Px(200.0),
                        ..default()
                    },
                    image: UiImage::new(icon),
                    ..default()
                });
            })
            .with_children(|parent| {
                parent.spawn(
                    TextBundle::from_section(
                        "made with bevy",
                        TextStyle {
                            font_size: 80.0,
                            color: WHITE.into(),
                            ..default()
                        },
                    ),
                );
            });
        commands.insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)));
    }


    fn countdown(
        mut commands: Commands,
        mut game_state: ResMut<NextState<GameState>>,
        time: Res<Time>,
        mut timer: Option<ResMut<SplashTimer>>,
        mut query: Query<Entity, With<UiImage>>,
        mut text_query: Query<Entity, With<Text>>
    ) {
        if timer.is_none() {
            return;
        }

            if let Some(mut timer) = timer {
                if timer.tick(time.delta()).finished() {
                    for image in &mut query {
                        game_state.set(GameState::Game);
                        commands.entity(image).despawn();
                        commands.remove_resource::<SplashTimer>();
                    }
                    for text in &mut text_query {
                        commands.entity(text).despawn();
                    }
                }
            } else {
                eprintln!("SplashTimer resource not found!");
            }
    }

    #[derive(Component)]
    struct OnGameScreen;

    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut background_color, selected) in &mut interaction_query {
            *background_color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
        menu_state.set(MenuState::Main);
    }

    fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        // Common style for all buttons on the screen
        let button_style = Style {
            width: Val::Px(250.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_icon_style = Style {
            width: Val::Px(30.0),
            // This takes the icons out of the flexbox flow, to be positioned exactly
            position_type: PositionType::Absolute,
            // The icon will be close to the left border of the button
            left: Val::Px(10.0),
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: 40.0,
            color: WHITE.into(),
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnMainMenuScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: CRIMSON.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        // Display the game name
                        parent.spawn(
                            TextBundle::from_section(
                                "Game Over",
                                TextStyle {
                                    font_size: 80.0,
                                    color: WHITE.into(),
                                    ..default()
                                },
                            )
                                .with_style(Style {
                                    margin: UiRect::all(Val::Px(50.0)),
                                    ..default()
                                }),
                        );

                        // Display three buttons for each action available from the main menu:
                        // - new game
                        // - settings
                        // - quit
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Play,
                            ))
                            .with_children(|parent| {
                                parent.spawn(ImageBundle {
                                    style: button_icon_style.clone(),
                                    ..default()
                                });
                                parent.spawn(TextBundle::from_section(
                                    "New Game",
                                    button_text_style.clone(),
                                ));
                            });
                    });
            });
    }

    fn menu_action(
        interaction_query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
        mut app_exit_events: EventWriter<AppExit>,
        mut menu_state: ResMut<NextState<MenuState>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut commands: Commands,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {

                match menu_button_action {
                    MenuButtonAction::Quit => {
                        app_exit_events.send(AppExit::Success);
                    }
                    MenuButtonAction::Play => {
                        game_state.set(GameState::Game);
                        menu_state.set(MenuState::Disabled);
                    }
                    MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                    MenuButtonAction::SettingsDisplay => {
                        menu_state.set(MenuState::SettingsDisplay);
                    }
                    MenuButtonAction::SettingsSound => {
                        menu_state.set(MenuState::SettingsSound);
                    }
                    MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                    MenuButtonAction::BackToSettings => {
                        menu_state.set(MenuState::Settings);
                    }
                }
            }
        }
    }

    // Generic system that takes a component as a parameter, and will despawn all entities with that component
    fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
        for entity in &to_despawn {
            commands.entity(entity).despawn_recursive();
        }
    }

    fn clear_after_game_over(
        mut commands: Commands,
        mut query: Query<Entity, With<SpawnTimer>>,
        mut enemy_query: Query<Entity, With<Enemy>>,
        mut score_counter_query: Query<Entity, With<ScoreCounter>>,
    ) {
        for (spawn_timer) in &mut query {
                commands.entity(spawn_timer).despawn();
        }
        for (enemy) in &mut enemy_query {
            commands.entity(enemy).despawn();
        }
        for (score_counter) in &mut score_counter_query {
            commands.entity(score_counter).despawn();
    }
    }
 