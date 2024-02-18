use specs::prelude::*;
use super::{Viewshed,Position, Map, Player};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl <'a> System<'a> for VisibilitySystem {
    type SystemData = (WriteExpect<'a, Map>,
                       Entities<'a>,
                       WriteStorage<'a, Viewshed>,
                       WriteStorage<'a, Position>,
                       ReadStorage<'a, Player>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, entities, mut viewshed, position,player) = data;

        for (entities, viewshed, position) in ( &entities, &mut viewshed, &position).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles = field_of_view(Point::new(position.x, position.y), viewshed.range, &*map);
                viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                //If this is the player, reveal what they can see
                let _p: Option<&Player> = player.get(entities);
                if let Some(_p) = _p {
                    for t in map.visible_tiles.iter_mut() {*t = false};
                    for visible in viewshed.visible_tiles.iter() {
                        let index = map.xy_index(visible.x, visible.y);
                        map.revealed_tiles[index] = true;
                        map.visible_tiles[index] = true;
                    }
                }
            }
        }
    }
}
