use rltk::{GameState, Rltk, Point};
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod map_indexing_system;
use map_indexing_system::MapIndexingamestateystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;
mod gui;
mod gamelog;
mod spawner;
mod inventory_system;
use inventory_system::{ ItemCollectionSystem, PotionUseSystem, ItemDropSystem };



#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn, ShowInventory, ShowDropItem }

pub struct State {
    pub ecs: World
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingamestateystem{};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);
        let mut potions = PotionUseSystem{};
        potions.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem{};
        drop_items.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, context : &mut Rltk) {
        context.cls();

        draw_map(&self.ecs, context);

        {
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            let map = self.ecs.fetch::<Map>();

            let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
            data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order) );
            for (pos, render) in data.iter() {
                let index = map.xy_index(pos.x, pos.y);
                if map.visible_tiles[index] { context.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
            }

            gui::draw_ui(&self.ecs, context);
        }

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, context);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, context);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDrinkPotion>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDrinkPotion{ potion: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, context);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem{ item: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .with_fullscreen(true)
        .build()?;
    context.with_post_scanlines(true);
    let mut gamestate = State {
        ecs: World::new(),
    };
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
    gamestate.ecs.register::<Item>();
    gamestate.ecs.register::<Potion>();
    gamestate.ecs.register::<InBackpack>();
    gamestate.ecs.register::<WantsToPickupItem>();
    gamestate.ecs.register::<WantsToDrinkPotion>();
    gamestate.ecs.register::<WantsToDropItem>();

    let map : Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let player_entity = spawner::player(&mut gamestate.ecs, player_x, player_y);

    gamestate.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gamestate.ecs, room);
    }

    gamestate.ecs.insert(map);
    gamestate.ecs.insert(Point::new(player_x, player_y));
    gamestate.ecs.insert(player_entity);
    gamestate.ecs.insert(RunState::PreRun);
    gamestate.ecs.insert(gamelog::GameLog{ entries : vec!["Welcome to Rusty Roguelike".to_string()] });

    rltk::main_loop(context, gamestate)
}