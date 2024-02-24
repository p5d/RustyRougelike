use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use super::{CombatStats, Player, Renderable, Name, Position, Viewshed, Monster, BlocksTile, Rect, Item, Potion, map::MAPWIDTH};

/// Spawns the player and returns his/her entity object.
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(Player{})
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Name{name: "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 5, power: 5 })
        .build()
}

const MAX_MONSTERS : i32 = 4;
const MAX_ITEMS : i32 = 2;

/// Fills a room with stuff!
pub fn spawn_room(ecs: &mut World, room : &Rect) {
    let mut monster_spawn_points : Vec<usize> = Vec::new();
    let mut item_spawn_points : Vec<usize> = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0 .. num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let index = (y * MAPWIDTH) + x;
                if !monster_spawn_points.contains(&index) {
                    monster_spawn_points.push(index);
                    added = true;
                }
            }
        }

        for _i in 0 .. num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let index = (y * MAPWIDTH) + x;
                if !item_spawn_points.contains(&index) {
                    item_spawn_points.push(index);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for index in monster_spawn_points.iter() {
        let x = *index % MAPWIDTH;
        let y = *index / MAPWIDTH;
        random_monster(ecs, x as i32, y as i32);
    }

    // Actually spawn the potions
    for index in item_spawn_points.iter() {
        let x = *index % MAPWIDTH;
        let y = *index / MAPWIDTH;
        health_potion(ecs, x as i32, y as i32);
    }
}

fn random_monster(ecs: &mut World, x: i32, y: i32) {

    fn orc (ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('o'), "Orc"); }
    fn goblin (ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('g'), "Goblin"); }
    fn hobgoblin (ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('h'), "Hobgoblin"); }
    fn rust_monster (ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('r'), "Rust Monster" );}
    fn ferris (ecs: &mut World, x: i32, y:i32) {monster(ecs, x, y, rltk::to_cp437('F'), "Ferris The Rustacean");}

    let roll :i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 5);
    }
    match roll {
        1 => { orc (ecs, x, y) }
        2 => { goblin (ecs, x, y) }
        3 => { hobgoblin(ecs, x, y)}
        4 => { rust_monster(ecs, x, y)}
        _ => {ferris(ecs, x, y)}
    }
}

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph : rltk::FontCharType, name : S) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Monster{})
        .with(Name{ name : name.to_string() })
        .with(BlocksTile{})
        .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
        .build();
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Health Potion".to_string() })
        .with(Item{})
        .with(Potion{ heal_amount: 8 })
        .build();
}