use bevy::ecs::schedule::ExecutorKind;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_ratatui::RataguiBackend;
use rand::seq::SliceRandom;
use rand::{self, thread_rng};
use std::iter::FromIterator;
use std::time::Duration;

use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Stylize, Terminal},
    widgets::{Block, Borders, Paragraph, Wrap},
};

#[derive(Clone, PartialEq, Debug, Hash, Eq, Copy)]
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

#[derive(Resource, Debug, Hash, Clone, PartialEq, Eq)]
struct GameState {
    grid: Vec<Vec<char>>,
    current_direction: Option<MoveDirection>,
    player_position: (i32, i32),
    scores: Scores,
    ghost_1_pos: (i32, i32),
    ghost_1_cur_dir: Option<MoveDirection>,
    num_moves_ghost_1_in_dir: i32,
    game_over: bool,
}

fn read_input(key: &str) -> Option<Command> {
    if key == "q" {
        return Some(Command::Quit);
    }
    if key == "w" {
        return Some(Command::Move(MoveDirection::Up));
    }
    if key == "a" {
        return Some(Command::Move(MoveDirection::Left));
    }
    if key == "s" {
        return Some(Command::Move(MoveDirection::Down));
    }
    if key == "d" {
        return Some(Command::Move(MoveDirection::Right));
    }
    if key == "r" {
        return Some(Command::Reset);
    }
    return None;
}

fn get_potential_moves(
    current_position: &(i32, i32),
    game_state: &GameState,
) -> Vec<((MoveDirection, (i32, i32)), char)> {
    [
        MoveDirection::Up,
        MoveDirection::Down,
        MoveDirection::Left,
        MoveDirection::Right,
    ]
    .map(|dir| {
        (
            dir.clone(),
            next_position(current_position, game_state, &Some(dir)),
        )
    })
    .map(|dir_coords| {
        (
            dir_coords.clone(),
            get_grid_cell_contents(&game_state.grid, dir_coords.1),
        )
    })
    .into_iter()
    .filter(|el| !['#', '|', '-', '_'].contains(&el.1))
    .collect::<Vec<((MoveDirection, (i32, i32)), char)>>()
}

fn opposite_direction(dir: &MoveDirection) -> &MoveDirection {
    match dir {
        MoveDirection::Down => &MoveDirection::Up,
        MoveDirection::Up => &MoveDirection::Down,
        MoveDirection::Left => &MoveDirection::Right,
        MoveDirection::Right => &MoveDirection::Left,
    }
}

fn ghost_move(game_state: &mut GameState) {
    if game_state.ghost_1_cur_dir.is_none() {
        return;
    }

    let current_ghost_position = game_state.ghost_1_pos;
    let cur_ghost_position_contents =
        get_grid_cell_contents(&game_state.grid, current_ghost_position);

    let mut next_ghost_position = next_position(
        &current_ghost_position,
        game_state,
        &game_state.ghost_1_cur_dir,
    );
    let mut next_ghost_position_contents =
        get_grid_cell_contents(&game_state.grid, next_ghost_position);

    if next_ghost_position_contents == 'K' {
        game_state.game_over = true;
        return;
    }

    let mut changed_direction = false;

    let mut potential_moves: Vec<((MoveDirection, (i32, i32)), char)> =
        get_potential_moves(&current_ghost_position, &game_state);

    if ['#', '|', '-', '_'].contains(&next_ghost_position_contents) {
        let mut next_move = potential_moves.choose(&mut thread_rng()).unwrap();
        if potential_moves.len() > 2 {
            potential_moves = potential_moves
                .clone()
                .into_iter()
                .filter(|move_info| {
                    &next_move.0 .0.clone() != opposite_direction(&move_info.0 .0.clone())
                })
                .collect::<Vec<((MoveDirection, (i32, i32)), char)>>();
        }
        next_move = potential_moves.choose(&mut thread_rng()).unwrap();

        next_ghost_position = next_move.0 .1;
        next_ghost_position_contents = next_move.1;
        game_state.ghost_1_cur_dir = Some(next_move.0 .0.clone());
        changed_direction = true;
    }
    // let mut logfile = File::create("/Users/jreed/pacman_rust/log.txt").unwrap();
    if next_ghost_position_contents == '•' {
        set_grid_cell(&mut game_state.grid, &next_ghost_position, '\u{1E43}');
        // let _ = logfile.write("moved into food\n".as_bytes());
    } else {
        set_grid_cell(&mut game_state.grid, &next_ghost_position, 'm');
        // let _ = logfile.write("moved into nothing\n".as_bytes());
    }

    if cur_ghost_position_contents == '\u{1E43}' {
        set_grid_cell(&mut game_state.grid, &current_ghost_position, '•');
        // let _ = logfile.write("moved FROM food\n".as_bytes());
    } else {
        set_grid_cell(&mut game_state.grid, &current_ghost_position, ' ');
        // let _ = logfile.write("moved FROM nothing\n".as_bytes());
    }
    game_state.ghost_1_pos = next_ghost_position;
    game_state.num_moves_ghost_1_in_dir = if changed_direction {
        0
    } else {
        game_state.num_moves_ghost_1_in_dir + 1
    };

    // let _ = logfile.write(
    //     format!("current ghost pos: {:?}\n current ghost contents: {:?}\n next ghost pos: {:?}\n next ghost contents: {:?}\n", current_ghost_position, cur_ghost_position_contents, next_ghost_position, next_ghost_position_contents).as_bytes(),
    // );
}

fn player_move(game_state: &mut GameState) {
    if game_state.current_direction.is_none() {
        return;
    }
    let current_player_position = game_state.player_position;
    let next_player_position = next_position(
        &current_player_position,
        game_state,
        &game_state.current_direction,
    );

    let next_player_position_contents =
        get_grid_cell_contents(&game_state.grid, next_player_position);

    if ['\u{1E43}', 'm'].contains(&next_player_position_contents) {
        game_state.game_over = true;
        return;
    }

    if next_player_position_contents == '#'
        || next_player_position_contents == '|'
        || next_player_position_contents == '_'
        || next_player_position_contents == '-'
    {
        // play_sound(SoundType::Oof, sink);
        return;
    }

    if next_player_position_contents == '•' {
        game_state.scores.current_score += 1;
    }

    set_grid_cell(&mut game_state.grid, &next_player_position, 'K');
    set_grid_cell(&mut game_state.grid, &current_player_position, ' ');
    game_state.player_position = next_player_position;
}

fn set_grid_cell(grid: &mut Vec<Vec<char>>, coords: &(i32, i32), contents: char) {
    grid[coords.1 as usize][coords.0 as usize] = contents;
}

fn get_grid_cell_contents(grid: &Vec<Vec<char>>, coords: (i32, i32)) -> char {
    grid[coords.1 as usize][coords.0 as usize]
}

fn next_position(
    current_position: &(i32, i32),
    game_state: &GameState,
    direction: &Option<MoveDirection>,
) -> (i32, i32) {
    match direction {
        Some(dir) => match dir {
            MoveDirection::Up => (current_position.0, std::cmp::max(0, current_position.1 - 1)),
            MoveDirection::Right => {
                if current_position.0 as usize == game_state.grid[10].len() - 1
                    && current_position.1 == 10
                {
                    return (0, 10);
                }
                return (
                    std::cmp::min(
                        game_state.grid[current_position.1 as usize].len() as i32 - 1,
                        current_position.0 + 1,
                    ),
                    current_position.1,
                );
            }
            MoveDirection::Down => (
                current_position.0,
                std::cmp::min(game_state.grid.len() as i32 - 1, current_position.1 + 1),
            ),
            MoveDirection::Left => {
                if current_position.0 == 0 && current_position.1 == 10 {
                    return ((game_state.grid[10].len() as i32) - 1, 10);
                }
                return (std::cmp::max(0, current_position.0 - 1), current_position.1);
            }
        },
        None => *current_position,
    }
}

fn text_input(
    mut gamestate: ResMut<GameState>,
    mut commands: Commands,
    mut evr_kbd: EventReader<KeyboardInput>,
) {
    for ev in evr_kbd.read() {
        // We don't care about key releases, only key presses
        if ev.state == ButtonState::Released {
            continue;
        }
        match &ev.logical_key {
            // Handle pressing Enter to finish the input
            // Handle key presses that produce text characters
            Key::Character(input) => {
                // Ignore any input that contains control (special) characters
                if input.chars().any(|c| c.is_control()) {
                    continue;
                }
                let cmd = read_input(input);
                match cmd {
                    Some(cmd) => match cmd {
                        Command::Move(dir) => {
                            gamestate.current_direction = Some(dir);
                        }
                        Command::Quit => {
                            break;
                        }
                        Command::Reset => (),
                    },
                    None => (),
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let grid = vec![
        vec![
            '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
            '_', '_',
        ],
        vec![
            '|', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', '|',
        ],
        vec![
            '|', ' ', '#', '#', ' ', '#', '#', '#', ' ', '#', ' ', '#', '#', '#', ' ', '#', '#',
            ' ', '|',
        ],
        vec![
            '|', ' ', '#', '#', ' ', '#', '#', '#', ' ', '#', ' ', '#', '#', '#', ' ', '#', '#',
            ' ', '|',
        ],
        vec![
            '|', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', '|',
        ],
        vec![
            '|', ' ', '#', '#', ' ', '#', ' ', ' ', '#', '#', '#', ' ', ' ', '#', ' ', '#', '#',
            ' ', '|',
        ],
        vec![
            '|', ' ', ' ', ' ', ' ', '#', ' ', ' ', ' ', '#', ' ', ' ', ' ', '#', ' ', ' ', ' ',
            ' ', '|',
        ],
        vec![
            '|', '#', '#', '#', ' ', '#', '#', '#', ' ', '#', ' ', '#', '#', '#', ' ', '#', '#',
            '#', '|',
        ],
        vec![
            '|', ' ', ' ', '#', ' ', '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#', ' ', '#', ' ',
            ' ', '|',
        ],
        vec![
            '|', '#', '#', '#', ' ', '#', ' ', '#', '#', ' ', '#', '#', ' ', '#', ' ', '#', '#',
            '#', '|',
        ],
        vec![
            ' ', ' ', 'K', ' ', 'm', ' ', ' ', '#', ' ', ' ', ' ', '#', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ',
        ],
        vec![
            '|', '#', '#', '#', ' ', '#', ' ', '#', '#', '#', '#', '#', ' ', '#', ' ', '#', '#',
            '#', '|',
        ],
        vec![
            '|', ' ', ' ', '#', ' ', '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#', ' ', '#', ' ',
            ' ', '|',
        ],
        vec![
            '|', '#', '#', '#', ' ', '#', ' ', '#', '#', '#', '#', '#', ' ', '#', ' ', '#', '#',
            '#', '|',
        ],
        vec![
            '|', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', '|',
        ],
        vec![
            '|', ' ', '#', '#', ' ', '#', '#', ' ', ' ', '#', ' ', ' ', '#', '#', ' ', '#', '#',
            ' ', '|',
        ],
        vec![
            '|', ' ', ' ', '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#', ' ',
            ' ', '|',
        ],
        vec![
            '|', '#', ' ', '#', ' ', '#', ' ', ' ', '#', '#', '#', ' ', ' ', '#', ' ', '#', ' ',
            ' ', '|',
        ],
        vec![
            '|', ' ', ' ', ' ', ' ', '#', ' ', ' ', ' ', '#', ' ', ' ', ' ', '#', ' ', ' ', ' ',
            ' ', '|',
        ],
        vec![
            '|', ' ', '#', '#', '#', '#', '#', '#', ' ', '#', ' ', '#', '#', '#', '#', '#', '#',
            ' ', '|',
        ],
        vec![
            '|', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', '|',
        ],
        vec![
            '-', '-', '-', '-', '-', '-', '-', '-', '-', '-', '-', '-', '-', '-', '-', '-', '-',
            '-', '-',
        ],
    ]
    .into_iter()
    .map(|row| {
        row.into_iter()
            .map(|cell| {
                if cell != ' ' {
                    return cell;
                }
                let num = rand::random::<f32>();
                if num < 0.75 {
                    return '•';
                }
                return cell;
            })
            .collect::<Vec<char>>()
    })
    .collect::<Vec<Vec<char>>>();
    App::new()
        .insert_resource(GameState {
            grid,
            current_direction: Some(MoveDirection::Right),
            player_position: (2, 10),
            scores: Scores {
                high_score: 0,
                current_score: 0,
            },
            ghost_1_cur_dir: Some(MoveDirection::Left),
            ghost_1_pos: (4, 10),
            num_moves_ghost_1_in_dir: 0,
            game_over: false,
        })
        .edit_schedule(FixedUpdate, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .add_plugins(DefaultPlugins)
        .init_resource::<BevyTerminal<RataguiBackend>>()
        //Initialize the ratatui terminal
        .add_plugins(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(
            Update,
            (
                text_input,
                move_people.run_if(on_timer(Duration::from_millis(500))),
                ui_example_system,
            ),
        )
        .run();
}

fn move_people(mut gamestate: ResMut<GameState>) {
    player_move(&mut gamestate);
    ghost_move(&mut gamestate);
}

// Render to the terminal and to egui , both are immediate mode
fn ui_example_system(
    gamestate: ResMut<GameState>,
    mut contexts: EguiContexts,
    mut termres: ResMut<BevyTerminal<RataguiBackend>>,
) {
    let grid = &gamestate.grid;
    termres
        .terminal
        .draw(|frame| {
            let areas =
                Layout::vertical(vec![Constraint::Length(1); grid.len()]).split(frame.area());

            // use the simpler short-hand syntax
            grid.iter().enumerate().for_each(|(idx, row)| {
                frame.render_widget(Paragraph::new(String::from_iter(row)).yellow(), areas[idx]);
            });
        })
        .expect("epic fail");

    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        //  ui.set_opacity(0.5);
        let huh = termres.terminal.backend_mut();
        ui.add(huh);
    });
}
// Create resource to hold the ratatui terminal
#[derive(Resource)]
struct BevyTerminal<RataguiBackend: ratatui::backend::Backend> {
    terminal: Terminal<RataguiBackend>,
}

// Implement default on the resource to initialize it
impl Default for BevyTerminal<RataguiBackend> {
    fn default() -> Self {
        let backend = RataguiBackend::new(100, 50);
        let terminal = Terminal::new(backend).unwrap();
        BevyTerminal { terminal }
    }
}
