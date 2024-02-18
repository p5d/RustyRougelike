use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use std::cmp::{min, max};
use super::{Position, Player, Viewshed, TileType, State, Map, RunState, CombatStats, WantsToMelee};

//Function to move the player
pub fn try_move_player (delta_x: i32, delta_y:i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>;
    let map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (entity, _player, position, viewshed) in (&entities, &player, &mut position, &mut viewshed),join(){
        if position.x + delta_x < 1 || position.x + delta_x > map.width-1 || position.y + delta_y < 1 || position.y + delta_y > map.height-1 { return; }
        let destination_index = map.xy_index(position.x + delta_x, position.y + delta_y);

        for potential_target in maps.tile_content[destination_index].iter(){

        }


// Min Max to prevent them from walking off screen
    for (entity, _players, position, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        if position.x + delta_x < 1 || position.x + delta_x > map.width -1 || pos.y + delta_y < 1 || pos.y + delta_y > map.height - 1 { return; }
        let destination_index = map.xy_index(position.x + delta_x, position.y + delta_y);

        for potential_target in map.tile_content[destination_index].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee.insert(entity, WantsToMelee{ target : *poetential_target }).expect("Add Target failed");
                }
            }
        }

        if !map.blocked[destination_index] {
            position.x = min(79, max(0, position.x + delta_x));
            position.y = min(49, max(0, position.y + delta_y));

            viewshed.dirty = true;
            let mut playerposition = ecs.write_resource::<Point>();
            playerposition.x = position.x;
            playerposition.y = position.y;

        }
    }
}

pub fn player_input(gamestate: &mut State, context: &mut Rltk) -> RunState {
    //Player movement
    match context.key {
        None => { return RunState::Paused } //Nothing happens
        Some(key) => match key {
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H |
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gamestate.ecs),

            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L |
            VirtualKeyCode::Right => try_move_player(1, 0 , &mut gamestate.ecs),

            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K |
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gamestate.ecs),

            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J |
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gamestate.ecs),

            //Diagonals
            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::Y => try_move_player(-1, 1, &mut gamestate.ecs),

            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::U => try_move_player(-1, -1, &mut gamestate.ecs),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::N => try_move_player(1, 1, &mut gamestate.ecs),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::B => try_move_player(1, -1, &mut gamestate.ecs),

            //Wait aka Do Nothing
            VirtualKeyCode::Key5 |
            VirtualKeyCode::Numpad5 => try_move_player(0, 0, &mut gamestate.ecs),
            _ => {return RunState::AwaitingInput } // All other unlisted keystrokes
        },
    }
    RunState::PlayerTurn
}
