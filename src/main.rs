use std::fmt::format;
use rltk::{GameState, Rltk, RGB, Point};
use specs::prelude::*;
mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rectangles;
pub use rectangles::*;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_systems;
use monster_ai_systems::MonsterAi;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;

//No Longer constantly running tick
#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn }

//Give the ECS a space to create aka a world
pub struct State {
    pub ecs: World
}

impl State{
    fn run_system(&mut self) {
        let mut visible = VisibilitySystem{};
        visible.run_now(&self.ecs);
        let mut mob = MonsterAi{};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        self.ecs.maintain();
    }
}


// Create a mutable game state
impl GameState for State {
    //Tick refreshes the screen
    fn tick(&mut self, context: &mut Rltk) {
        context.cls();
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate{
            RunState::PreRun => {
                self.run_system();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, context);
            }
            RunState::PlayerTurn => {
                self.run_system();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_system();
                newrunstate = RunState::AwaitingInput;
            }
        }
        {
            let mu runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);

        draw_map(&self.ecs, context);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (position, render) in (&positions, &renderables).join() {
            let index = map.xy_index(position.x, position.y);
            if map.visible_tiles[index] {context.set(position.x, position.y, render.foreground, render.background, render.glyph) }
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50() //Size of window
        .with_fitscreen(true)
        .with_title("Some Epic RPG Name Here")//Obvious Title is Obvious
        .build()?;

    let mut gamestate = State {
        ecs: World::new(),
    };

    // Register the components created to the ECS
    gamestate.ecs.register::<Position>();
    gamestate.ecs.register::<Renderable>();
    gamestate.ecs.register::<Player>();
    gamestate.ecs.register::<Viewshed>();
    gamestate.ecs.register::<Monster>();
    gamestate.ecs.register::<Name>();
    gamestate.ecs.register::<BlocksTile>();
    gamestate.ecs.register::<CombatStats>();
    gamestate.ecs.register::<WantsToMelee>();
    gamestate.ecs.register::<SufferDamage>();

    let map = Map::random_room_dungeon();
    let (player_x, player_y) = map.rooms[0].center();

    let player_entity = gamestate.ecs
        .create_entity()
        .with(Position {x: player_x, y: player_y})
        .with(Renderable {
            glyph: rltk::to_cp437(@),
            foreground: rltk::named(rltk::GREENYELLOW),
            background: rltk::named(rltk::BLACK),
        })
        .with(Player{})
        .with(Viewshed{visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name{name : "Player".to_string() })
        .with(CombatStats{ max_hp: 30, current_hp: 30, defense: 2, power: 5})
        .build();

    let mut rng = rltk::RandomNumberGenerator::new();
    for (i,room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph: rltk::FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => { glyph = rltk::to_cp437('g'); name = "Goblin".to_string(); }
            _ => { glyph = rltk::to_cp437('o'); name = "Orc".to_string();}
        }

        gamestate.ecs.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                foreground: RGB::named(rltk::RED),
                background: RGB::named(rltk::BLACK),
            })
            .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
            .with(Monster{})
            .with(Name{name: format!("{} #{}", &name, i) })
            .with(BlocksTile{})
            .with(CombatStats{max_hp: 5, current_hp: 5, defense: 3, power: 2})
            .build();
    }

    gamestate.ecs.insert(map);
    gamestate.ecs.insert(Point::new(player_x, player_y));
    gamestate.ecs.insert(player_entity);
    gamestate.ecs.insert(RunState::PreRun);

    rltk::main_loop(context, gamestate)
}
