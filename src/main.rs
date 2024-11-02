use bevy::time::common_conditions::on_timer;
use bevy::utils::HashSet;
use rand::{self};
use std::time::Duration;

use bevy::sprite::{Wireframe2dConfig, Wireframe2dPlugin};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Stylize, Terminal},
    widgets::{Block, Borders, Paragraph, Wrap},
};

#[derive(States, Clone, PartialEq, Debug, Hash, Eq, Copy)]
enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq, Copy)]
struct Scores {
    high_score: i32,
    current_score: i32,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq, Copy)]
enum Command {
    Quit,
    Move(MoveDirection),
    Reset,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Playing,
    Restarting,
    GameOver,
}

#[derive(Resource, Debug, Hash, Clone, PartialEq, Eq)]
struct GameState {
    current_direction: Option<MoveDirection>,
    player_position: (i32, i32),
    scores: Scores,
    ghost_1_pos: (i32, i32),
    ghost_1_cur_dir: Option<MoveDirection>,
    num_moves_ghost_1_in_dir: i32,
}

fn read_input(keys: Res<ButtonInput<KeyCode>>) -> Option<Command> {
    if keys.just_released(KeyCode::KeyQ) {
        return Some(Command::Quit);
    }
    if keys.just_released(KeyCode::KeyW) {
        return Some(Command::Move(MoveDirection::Up));
    }
    if keys.just_released(KeyCode::KeyA) {
        return Some(Command::Move(MoveDirection::Left));
    }
    if keys.just_released(KeyCode::KeyS) {
        return Some(Command::Move(MoveDirection::Down));
    }
    if keys.just_released(KeyCode::KeyD) {
        return Some(Command::Move(MoveDirection::Right));
    }
    if keys.just_released(KeyCode::KeyR) {
        return Some(Command::Reset);
    }
    return None;
}

#[derive(Debug)]
struct Update {
    coord: (i32, i32),
    value: Vec<(Transform, Entity)>,
}

#[derive(Resource, Debug)]
struct Updates(pub Vec<Update>);

fn update_resourcemap(mut resource_map: ResMut<ResourceMap>, mut updates: ResMut<Updates>) {
    updates.0.drain(..).for_each(|update| {
        resource_map.0[update.coord.1 as usize][update.coord.0 as usize] = update.value;
    });
}

fn player_move(
    mut game_state: ResMut<GameState>,
    next_pos: ResMut<NextPacmanPosition>,
    tilemap: Res<Tilemap>,
    mut next_state: ResMut<NextState<AppState>>,
    resourcemap: Res<ResourceMap>,
    mut commands: Commands,
    window_query: Query<&Window>,
    mut updates: ResMut<Updates>,
) {
    let player_entity = &resourcemap.0[game_state.player_position.1 as usize]
        [game_state.player_position.0 as usize];
    if next_pos.pos.x as i32 == game_state.player_position.0
        && next_pos.pos.y as i32 == game_state.player_position.1
    {
        return;
    }
    if player_entity.len() == 0 {
        panic!("The player didnt exist?!");
    }

    if next_pos.contents.contains(&'m') {
        next_state.set(AppState::GameOver);
        return;
    }

    if vec!['#', '_', '-', '|']
        .iter()
        .collect::<HashSet<&char>>()
        .intersection(&next_pos.contents.iter().collect::<HashSet<&char>>())
        .collect::<Vec<&&char>>()
        .len()
        > 0
    {
        next_state.set(AppState::Playing);
        return;
    }

    if next_pos.contents.contains(&'•') {
        game_state.scores.current_score += 1;
        resourcemap.0[next_pos.pos.y as usize][next_pos.pos.x as usize]
            .iter()
            .for_each(|t: &(Transform, Entity)| {
                commands.entity(t.1).despawn();
            });
    }

    updates.0.push(Update {
        coord: game_state.player_position,
        value: vec![],
    });

    // create the new transform and entity
    let window = window_query.single();
    let border_len = (window.height()) / (tilemap.0.len() as f32);
    let top_left = Vec3::new(-window.width() / 2.0, window.height() / 2.0, 0.0);
    let x = next_pos.pos.x as f32 * border_len + (border_len / 2.0);
    let y = -(next_pos.pos.y * border_len) - (border_len / 2.0);
    let transform = Transform {
        translation: top_left + Vec3::new(x, y, 0.0),
        rotation: calc_rotation(game_state.current_direction.unwrap()),
        ..default()
    };

    commands.entity(player_entity[0].1).insert(transform);
    updates.0.push(Update {
        coord: (next_pos.pos.x as i32, next_pos.pos.y as i32),
        value: vec![(transform, player_entity[0].1)],
    });

    game_state.player_position = (next_pos.pos.x as i32, next_pos.pos.y as i32);
    next_state.set(AppState::Playing);
}

fn ghost1_move(
    mut game_state: ResMut<GameState>,
    next_pos: ResMut<NextGhost1Position>,
    tilemap: Res<Tilemap>,
    mut next_state: ResMut<NextState<AppState>>,
    resourcemap: Res<ResourceMap>,
    mut commands: Commands,
    window_query: Query<&Window>,
    mut updates: ResMut<Updates>,
) {
    let ghost_entity =
        &resourcemap.0[game_state.ghost_1_pos.1 as usize][game_state.ghost_1_pos.0 as usize];
    if next_pos.pos.x as i32 == game_state.ghost_1_pos.0
        && next_pos.pos.y as i32 == game_state.ghost_1_pos.1
    {
        return;
    }
    if ghost_entity.len() == 0 {
        panic!("The ghost didnt exist?!");
    }

    if next_pos.contents.contains(&'K') {
        next_state.set(AppState::GameOver);
        return;
    }

    if vec!['#', '_', '-', '|']
        .iter()
        .collect::<HashSet<&char>>()
        .intersection(&next_pos.contents.iter().collect::<HashSet<&char>>())
        .collect::<Vec<&&char>>()
        .len()
        > 0
    {}

    if next_pos.contents.contains(&'•') {
        game_state.scores.current_score += 1;
        resourcemap.0[next_pos.pos.y as usize][next_pos.pos.x as usize]
            .iter()
            .for_each(|t: &(Transform, Entity)| {
                commands.entity(t.1).despawn();
            });
    }

    updates.0.push(Update {
        coord: game_state.player_position,
        value: vec![],
    });

    // create the new transform and entity
    let window = window_query.single();
    let border_len = (window.height()) / (tilemap.0.len() as f32);
    let top_left = Vec3::new(-window.width() / 2.0, window.height() / 2.0, 0.0);
    let x = next_pos.pos.x as f32 * border_len + (border_len / 2.0);
    let y = -(next_pos.pos.y * border_len) - (border_len / 2.0);
    let transform = Transform {
        translation: top_left + Vec3::new(x, y, 0.0),
        rotation: calc_rotation(game_state.current_direction.unwrap()),
        ..default()
    };

    commands.entity(ghost_entity[0].1).insert(transform);
    updates.0.push(Update {
        coord: (next_pos.pos.x as i32, next_pos.pos.y as i32),
        value: vec![(transform, ghost_entity[0].1)],
    });

    game_state.player_position = (next_pos.pos.x as i32, next_pos.pos.y as i32);
    next_state.set(AppState::Playing);
}

#[rustfmt::skip]
fn create_tilemap() -> Tilemap {
    Tilemap(vec![
        vec![vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_'], vec!['_']],
        vec![vec!['|'], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec!['|']],
        vec![vec!['|'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['|']],
        vec![vec!['|'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['|']],
        vec![vec!['|'], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec!['|']],
        vec![vec!['|'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['|']],
        vec![vec!['|'], vec![' '], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec![' '], vec!['|']],
        vec![vec!['|'], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec!['|']],
        vec![vec!['|'], vec!['Z'], vec!['Z'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['Z'], vec!['Z'], vec!['|']],
        vec![vec!['|'], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec!['|']],
        vec![vec![' '], vec![' '], vec!['K'], vec![' '], vec![' '], vec![' '], vec![' '], vec!['#'], vec!['#'], vec!['m'], vec!['#'], vec!['#'], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' ']],
        vec![vec!['|'], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec!['|']],
        vec![vec!['|'], vec![' '], vec![' '], vec!['#'], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec!['#'], vec![' '], vec![' '], vec!['|']],
        vec![vec!['|'], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec!['|']],
        vec![vec!['|'], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec!['|']],
        vec![vec!['|'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec![' '], vec!['|']],
        vec![vec!['|'], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec!['|']],
        vec![vec!['|'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec![' '], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec![' '], vec!['#'], vec![' '], vec!['#'], vec![' '], vec![' '], vec!['|']],
        vec![vec!['|'], vec![' '], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec!['#'], vec![' '], vec![' '], vec![' '], vec![' '], vec!['|']],
        vec![vec!['|'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['#'], vec![' '], vec!['#'], vec!['#'], vec!['#'], vec!['#'], vec!['#'], vec!['#'], vec![' '], vec!['|']],
        vec![vec!['|'], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec![' '], vec!['|']],
        vec![vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-'], vec!['-']],
    ]
    .into_iter()
    .map(|row| {
        row.into_iter().map(|cell| {  
            if cell[0] != ' ' { 
                return cell;
            }
            let num = rand::random::<f32>();
            if num < 0.75 { 
                return vec!['•'];
            }
            return cell;
        })
        .collect::<Vec<Vec<char>>>()
    })
    .collect::<Vec<Vec<Vec<char>>>>())
}

fn start_state() -> GameState {
    return GameState {
        current_direction: None,
        player_position: (2, 10),
        scores: Scores {
            high_score: 0,
            current_score: 0,
        },
        ghost_1_cur_dir: Some(MoveDirection::Left),
        ghost_1_pos: (9, 10),
        num_moves_ghost_1_in_dir: 0,
    };
}

fn dir_to_int(dir: MoveDirection) -> i32 {
    match dir {
        MoveDirection::Down => 0,
        MoveDirection::Right => 1,
        MoveDirection::Up => 2,
        MoveDirection::Left => 3,
    }
}

fn calc_rotation(next_dir: MoveDirection) -> Quat {
    let next_dir_int = dir_to_int(next_dir);

    return Quat::from_rotation_z(((next_dir_int) as f32 * 90.0).to_radians());
}

fn text_input(
    mut gamestate: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let cmd = read_input(keys);
    match cmd {
        Some(cmd) => match cmd {
            Command::Move(dir) => {
                gamestate.current_direction = Some(dir);
            }
            Command::Quit => next_state.set(AppState::GameOver),
            Command::Reset => {
                next_state.set(AppState::Restarting);
            }
        },
        None => (),
    }
}

#[derive(Component)]
struct MyCameraMarker;

#[derive(Component)]
struct Pacman;

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    query: Query<(Entity, &Mesh2dHandle), With<Pacman>>,
    mut commands: Commands,
    mut timer_query: Query<&mut AnimationTimer>,
    mut pacman_meshes: ResMut<PacmanMeshes>,
) {
    let mut timer = timer_query.single_mut();
    timer.tick(time.delta());
    if timer.just_finished() {
        let (entity, handle) = query.single();
        if handle.id() == pacman_meshes.0[3].id() {
            pacman_meshes.0.reverse();
            commands.entity(entity).insert(pacman_meshes.0[0].clone());
        } else {
            let current_index = pacman_meshes
                .0
                .iter()
                .position(|m| m.id() == handle.id())
                .unwrap();
            commands
                .entity(entity)
                .insert(pacman_meshes.0[current_index + 1].clone());
        }
    }
}

#[derive(Resource)]
struct ResourceMap(pub Vec<Vec<Vec<(Transform, Entity)>>>);

#[derive(Resource)]
struct PacmanMeshes(pub Vec<Mesh2dHandle>);

fn create_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
    tilemap: Res<Tilemap>,
    mut resourcemap: ResMut<ResourceMap>,
    mut pacmans: ResMut<PacmanMeshes>,
) {
    let window = window.single();

    let border_len = (window.height()) / (tilemap.0.len() as f32);

    let starting_mesh = Mesh2dHandle(meshes.add(CircularSector::new(border_len / 3.0, 3.5)));
    pacmans.0.append(&mut vec![
        Mesh2dHandle(meshes.add(CircularSector::new(border_len / 3.0, 2.5))),
        Mesh2dHandle(meshes.add(CircularSector::new(border_len / 3.0, 2.65))),
        Mesh2dHandle(meshes.add(CircularSector::new(border_len / 3.0, 2.8))),
        starting_mesh.clone(),
    ]);
    let wall = Mesh2dHandle(meshes.add(Rectangle::new(border_len, border_len)));
    let pacman = starting_mesh.clone();
    let food = Mesh2dHandle(meshes.add(Circle {
        radius: (border_len / 10.0),
    }));

    let top_left = Vec3::new(-window.width() / 2.0, window.height() / 2.0, 0.0);
    let yellow = Color::linear_rgb(255.0, 255.0, 0.0);
    let blue = Color::linear_rgb(0.0, 0.0, 255.0);
    let mesh_default = default();
    let blue_handle = materials.add(blue);
    let yellow_handle = materials.add(yellow);

    resourcemap.0 = tilemap
        .0
        .iter()
        .enumerate()
        .map(|(row_idx, row)| {
            row.iter()
                .enumerate()
                .map(|(col_idx, tile)| {
                    let cell = tile[0];
                    let x = col_idx as f32 * border_len + (border_len / 2.0);
                    let y = -(row_idx as f32 * border_len) - (border_len / 2.0);
                    let Some((mesh, material, transform)) = (match cell {
                        '|' | '_' | '-' | '#' => Some((
                            wall.clone(),
                            blue_handle.clone(),
                            Transform::from_translation(top_left + Vec3::new(x, y, 0.0)),
                        )),
                        '•' => Some((
                            food.clone(),
                            yellow_handle.clone(),
                            Transform::from_translation(top_left + Vec3::new(x, y, -1.0)),
                        )),
                        'K' => Some((
                            pacman.clone(),
                            yellow_handle.clone(),
                            Transform {
                                translation: top_left + Vec3::new(x, y, 0.0),
                                rotation: calc_rotation(MoveDirection::Right),
                                ..default()
                            },
                        )),
                        _ => None,
                    }) else {
                        return vec![];
                    };
                    let entity = MaterialMesh2dBundle {
                        mesh,
                        material,
                        transform,
                        ..mesh_default
                    };
                    if cell == 'K' {
                        let entity_commands = commands.spawn((
                            AnimationTimer(Timer::from_seconds(0.0625, TimerMode::Repeating)),
                            entity,
                            Pacman,
                        ));
                        return vec![(transform, entity_commands.id())];
                    } else {
                        let entity_commands = commands.spawn(entity);
                        return vec![(transform, entity_commands.id())];
                    }
                })
                .collect::<Vec<Vec<(Transform, Entity)>>>()
        })
        .collect::<Vec<Vec<Vec<(Transform, Entity)>>>>();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        MyCameraMarker,
    ));
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<Wireframe2dConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
        wireframe_config.default_color = Color::srgba(255.0, 255.0, 255.0, 1.0).into();
    }
}

#[derive(Resource)]
struct Tilemap(pub Vec<Vec<Vec<char>>>);

#[derive(Resource)]
struct NextPacmanPosition {
    pub pos: Vec3,
    pub contents: Vec<char>,
}

#[derive(Resource)]
struct NextGhost1Position {
    pub pos: Vec3,
    pub contents: Vec<char>,
}
use bevy::input::common_conditions::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .insert_resource(start_state())
        .insert_resource(create_tilemap())
        .insert_resource(Updates(vec![]))
        .insert_resource(PacmanMeshes(vec![]))
        .insert_resource(NextPacmanPosition {
            pos: Vec3 {
                x: 2.0,
                y: 10.0,
                z: 0.0,
            },
            contents: vec![' '],
        })
        .insert_resource(ResourceMap(Vec::new()))
        .insert_state(AppState::Playing)
        .add_systems(Startup, (setup_camera, create_resources))
        .add_systems(
            Update,
            (
                toggle_wireframe,
                text_input,
                animate_sprite,
                (update_next_position, player_move, update_resourcemap)
                    .chain()
                    .run_if(on_timer(Duration::from_millis(500))),
            ),
        )
        .run();
}

fn update_next_position(
    tilemap: Res<Tilemap>,
    game_state: Res<GameState>,
    mut next_pos: ResMut<NextPacmanPosition>,
) {
    next_pos.pos = get_next_position(
        tilemap.as_ref(),
        game_state.player_position,
        game_state.current_direction,
    );
    next_pos.contents = tilemap.0[next_pos.pos.y as usize][next_pos.pos.x as usize].clone();
}

fn update_next_position_ghost1(
    tilemap: Res<Tilemap>,
    game_state: Res<GameState>,
    mut next_pos: ResMut<NextGhost1Position>,
) {
    next_pos.pos = get_next_position(
        tilemap.as_ref(),
        game_state.ghost_1_pos,
        game_state.ghost_1_cur_dir,
    );
    next_pos.contents = tilemap.0[next_pos.pos.y as usize][next_pos.pos.x as usize].clone();
}

fn get_next_position(
    tilemap: &Tilemap,
    position: (i32, i32),
    direction: Option<MoveDirection>,
) -> Vec3 {
    let mut next_pos = Vec3 {
        x: position.0 as f32,
        y: position.1 as f32,
        z: 0.0,
    };
    match direction {
        Some(dir) => match dir {
            MoveDirection::Up => {
                next_pos.y = std::cmp::max(0, next_pos.y as i32 - 1) as f32;
            }
            MoveDirection::Left => {
                next_pos.x = std::cmp::max(0, next_pos.x as i32 - 1) as f32;
            }
            MoveDirection::Down => {
                next_pos.y =
                    std::cmp::min(tilemap.0.len() as i32 - 1, next_pos.y as i32 + 1) as f32;
            }
            MoveDirection::Right => {
                next_pos.x = std::cmp::min(
                    tilemap.0[next_pos.y as usize].len() as i32 - 1,
                    next_pos.x as i32 + 1,
                ) as f32;
            }
        },
        None => (),
    }
    return next_pos;
}
