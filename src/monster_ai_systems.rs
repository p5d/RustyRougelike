use specs::prelude::*;
use super::{Viewshed, Monster, Name, Map, Position, WantsToMelee, RunState};
use rltk::{Point};

pub struct MonsterAi {}

impl<'a> System<'a> for MonsterAi {
    #[allow(clippy::type_complexity)]
    type  SystemData = ( WriteExpect<'a, Map>,
                         ReadExpect<'a, Point>,
                         ReadExpect<'a, Entity>,
                         ReadExpect<'a, RunState>,
                         Entities<'a>,
                         WriteStorage<'a, Viewshed>,
                         ReadStorage<'a, Monster>,
                         WriteStorage<'a, Position>,
                         WriteStorage<'a, WantsToMelee>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_position, player_entity, runstate, entities, mut viewshed, monster, mut position, mut wants_to_melee) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, mut viewshed, _monster, mut position) in (&entities, &mut viewshed, &monster, &mut position).join() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(position.x, position.y), *player_position);
            if distance < 1.5 {
                wants_to_melee.insert(entity, WantsToMelee { target: *player_entity }).expect("Unable to insert attack");
            }
            else if viewshed.visible_tiles.contains(&*player_entity) {
                //Path to the player
                let path = rltk::a_star_search(
                    map.xy_index(position.x, position.y),
                    map.xy_index(player_position.x, player_position.y),
                    &mut *map
                );
                if path.success && path.steps.len()>1 {
                    let mut index = map.xy_index(position.x, position.y);
                    map.blocked[index] = false;
                    position.x = path.steps[1] as i32 % map.width;
                    position.y = path.steps[1] as i32 / map.width;
                    index = map.xy_index(position.x, position.y);
                    map.blocked[index] = true;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
