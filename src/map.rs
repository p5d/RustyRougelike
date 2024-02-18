use rltk::{RGB, Rltk, RandomNumberGenerator, BaseMap, SmallVec, Algorithm2D, Point};
use super::{Rectangles};
use std::cmp::{max, min};
use specs::prelude::*;

const MAPWIDTH: usize = 80;
const MAPHEIGHT: usize = 43;
const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType{
    Wall,Floor
}

#[derive(Default)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rectangles>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub tile_content : Vec<Vec<Entity>>

}

impl Map {
    pub fn xy_index(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }


    fn apply_room_to_map(&mut self, room: &Rectangles) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let index = self.xy_index(x, y);
                self.tiles[index] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let index = self.xy_index(x, y);
            if index > 0 && index < self.width as usize * self.height as usize {
                self.tiles[index as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnels(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2) ..=max(y1, y2) {
            let index = self.xy_index(x, y);
            if index > 0 && index < self.width as usize * self.height as usize {
                self.tiles[index as usize] = TileType::Floor;
            }
        }
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width -1 || y < 1 || y > self.height -1 {return false; }
        let index = self.xy_index(x,y);
        !self.blocked[index]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }



    //Randomly populates rooms and corridors for a dungeon.
    pub fn random_room_dungeon() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content : vec![Vec::new(); MAPCOUNT]
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let width = rng.range(MIN_SIZE, MAX_SIZE);
            let height = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - width - 1) - 1;
            let y = rng.roll_dice(1, 50 - height - 1) - 1;
            let new_room = Rectangles::new(x, y, width, height);
            let mut no_overlap = true;

            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { no_overlap = false }
            }
            if no_overlap {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (previous_x, previous_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(previous_x, new_x, previous_y);
                        map.apply_vertical_tunnels(previous_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnels(previous_y, new_y, previous_x);
                        map.apply_horizontal_tunnel(previous_x, new_x, new_y);
                    }
                }
                map.rooms.push(new_room);
            }
        }
        map
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, index: usize) -> bool {
        self.tiles[index] == TileType::Wall
    }


    fn get_pathing_distance(&self, index1: usize, index2: usize) -> f32 {
        let width = self.width as usize;
        let path1 = Point::new(index1 % width, index1 / width);
        let path2 = Point::new(index2 % width, index2 / width);
        rltk::DistanceAlg::Pythagoras.distance2d(path1, path2)
    }

    fn get_available_exits(&self, index:usize) -> rltk::SmallVec<[(usize,f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = index as i32 % self.width;
        let y = index as i32 / self.width;
        let width = self.width as usize;

        //Cardinal Directions
        if self.is_exit_valid(x-1, y) {exits.push((index -1, 1.0))};
        if self.is_exit_valid(x+1, y) {exits.push((index +1, 1.0))};
        if self.is_exit_valid(x, y-1) {exits.push((index - width, 1.0))};
        if self.is_exit_valid(x, y+1) {exits.push((index + width , 1.0))};

        //Diagonals
        if self.is_exit_valid(x-1, y-1) {exits.push(((index - width) -1, 1.45))};
        if self.is_exit_valid(x+1, y-1) {exits.push(((index - width) +1, 1.45))};
        if self.is_exit_valid(x-1, y+1) {exits.push(((index + width) -1, 1.45))};
        if self.is_exit_valid(x+1, y+1) {exits.push(((index + width) -1, 1.45))};

        exits
    }
}


impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

pub fn draw_map(ecs: &World, context: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;
    for (index, tile) in map.tiles.iter().enumerate() {

        if map.revealed_tiles[index] {
            let glyph;
            let mut foreground;
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    foreground = RGB::from_u8(227, 15, 94)
                    }
                TileType::Wall => {
                    glyph = rltk::to_cp437('#');
                    foreground = RGB::from_u8(255, 170, 51);
                }
            }
            if !map.visible_tiles[index] { foreground = foreground.to_greyscale() }
            context.set(x, y, foreground, RGB::from_f32(0., 0., 0.), glyph);
        }
        // Move the coordinates
        x += 1;
        if x > MAPWIDTH as i32 - 1 {
            x = 0;
            y += 1;
        }
    }
}
