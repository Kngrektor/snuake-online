use std::collections::HashMap;
use std::collections::VecDeque;
use std::num::NonZeroUsize;

use crate::entity::*;
use crate::util::*;

extern crate rand;
use rand::prelude::*;

extern crate serde;
use serde::{Deserialize, Serialize};

// ++++++++++++++++++++++++++++++++++
// + default implementation of Prop +
// ++++++++++++++++++++++++++++++++++

enum Food {
    GrowFood,
    BadFood,
}

impl Prop for Food {
    fn collision_result(&self) -> CollisionResult {
        match self {
            Food::BadFood => CollisionResult::RemoveBoth,
            Food::GrowFood => CollisionResult::RemoveSelf,
        }
    }

    fn collision_events(&self, id: SnakeID) -> Vec<SnakeEvent> {
        match self {
            Food::BadFood => vec![SnakeEvent {
                id: id,
                cmd: Cmd::Kill,
            }],
            Food::GrowFood => vec![
                SnakeEvent {
                    id: id,
                    cmd: Cmd::Grow(NonZeroUsize::one()),
                },
                SnakeEvent {
                    id: id,
                    cmd: Cmd::GiveScore(1),
                },
            ],
        }
    }

    fn get_timer(&self) -> Option<Timer> {
        Some(Timer::new(20))
    }

    fn id(&self) -> ID {
        match self {
            Food::GrowFood => 0,
            Food::BadFood => 1,
        }
    }
}

fn food_spawner() -> Box<Prop> {
    let food: Vec<Box<Prop>> =
        vec![Box::new(Food::GrowFood), Box::new(Food::BadFood)];

    let mut rng = rand::thread_rng();
    food.into_iter().choose(&mut rng).unwrap()
}

// ++++++++
// + Grid +
// ++++++++

struct Grid {
    grid: Vec<Vec<Option<Entity>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridData {
    pub rows: u32,
    pub cols: u32,
    pub tags: Vec<Vec<Tag>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameFrom {
    Real(((u32, u32), (u32, u32))),
    Dummy(((u32, u32), (u32, u32))),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    pub came_from_heads: HashMap<SnakeID, CameFrom>,
    pub came_from_tails: HashMap<SnakeID, CameFrom>,
    pub grid_data: GridData,
}

impl Grid {
    fn new(rows: usize, cols: usize) -> Self {
        let mut grid = Vec::new();

        for i in 0..rows {
            grid.push(Vec::new());

            for _ in 0..cols {
                grid[i].push(None);
            }
        }

        Grid { grid: grid }
    }

    fn rows(&self) -> usize {
        self.grid.len()
    }

    fn cols(&self) -> usize {
        self.grid[0].len()
    }

    fn add(&mut self, idx: Index2D, e: Entity) {
        let (i, j) = idx.get();
        self.grid[i][j] = Some(e);
    }

    fn remove(&mut self, idx: Index2D) {
        let (i, j) = idx.get();
        self.grid[i][j] = None
    }

    fn index_of_next_vacant(&self, idx: Index2D) -> Option<Index2D> {
        let (start, end) = idx.get();

        for i in start..self.rows() {
            for j in end..self.cols() {
                if self.grid[i][j].is_none() {
                    return Some(Index2D::new(i, j));
                }
            }
        }

        for i in 0..self.rows() {
            for j in 0..self.cols() {
                if self.grid[i][j].is_none() {
                    return Some(Index2D::new(i, j));
                }
            }
        }

        None
    }

    fn index_of_rand_vacant(&self) -> Option<Index2D> {
        let pos = Index2D::rand_in_range((0, self.rows()), (0, self.cols()));
        self.index_of_next_vacant(pos)
    }

    fn data(&self) -> GridData {
        let mut tags = vec![Vec::with_capacity(self.cols()); self.rows()];
        for i in 0 .. self.rows() {
            for j in 0 .. self.cols() {
                let x = self.grid[i][j]
                    .as_ref()
                    .map(|x| x.tag())
                    .unwrap_or(Tag { kind: Kind::None, id: 0 });

                tags[i].push(x);
            }
        }

        GridData {
            rows: self.rows() as u32,
            cols: self.cols() as u32,
            tags: tags,
        }
    }
}

// +++++++++
// + Snake +
// +++++++++

struct Snake {
    id: SnakeID,
    pos: Index2D,
    prev_last: Index2D,
    is_dead: bool,
    death_timer: Option<Timer>,
    spawn_timer: Timer,
    score: usize,
    body: VecDeque<Index2D>,
    grow_count: Option<NonZeroUsize>,
    curr_dir: Direction,
    next_dir: Option<Direction>,
    came_from_head: Option<CameFrom>,
    came_from_tail: Option<CameFrom>,
}

impl Snake {
    fn new(id: SnakeID, pos: Index2D) -> Self {
        let mut snake = Snake {
            id: id,
            pos: pos,
            prev_last: pos,
            is_dead: false,
            death_timer: None,
            spawn_timer: Timer::new(5),
            score: 0,
            body: VecDeque::new(),
            grow_count: None,
            curr_dir: Direction::rand(),
            next_dir: None,
            came_from_head: None,
            came_from_tail: None,
        };

        snake.grow(NonZeroUsize::one());
        snake
    }

    fn give_buff(&mut self, _buff: Box<Buff>) {}

    fn give_score(&mut self, score: i32) {
        let tmp = self.score as i32 + score;
        if tmp < 0 {
            self.score = 0;
        } else {
            self.score = tmp as usize;
        }
    }

    fn tick(&mut self, grid: &mut Grid) {
        if !self.spawn_timer.is_done() {
            self.spawn_timer.tick();
        }

        match self.death_timer.as_mut() {
            Some(timer) => {
                if !timer.is_done() {
                    timer.tick();

                    let first = self.body.pop_front();
                    first.map(|first| {
                        grid.remove(first);

                        let first = first.get_u32();

                        if let Some(new_first) = self.body.front() {
                            let x = CameFrom::Real((new_first.get_u32(), first));
                            self.came_from_tail = Some(x);
                        } else {
                            let prev_last = self.prev_last;
                            let x = CameFrom::Dummy((prev_last.get_u32(), first));
                            self.came_from_tail = Some(x);
                        }
                    });
                }
            }

            _ => (),
        }
    }

    fn should_spawn(&self) -> bool {
        self.death_timer
            .as_ref()
            .map(|x| x.is_done())
            .unwrap_or(false)
    }

    fn is_mortal(&self) -> bool {
        self.spawn_timer.is_done() && !self.is_dead
    }

    fn remove_head(&mut self, grid: &mut Grid) {
        if self.spawn_timer.is_done() {
            grid.remove(self.pos);
        }
    }

    fn tick_head(&mut self, grid: &mut Grid) {
        if !self.spawn_timer.is_done() {
            return;
        }

        let prev_pos = self.pos.get_u32();

        // move head
        self.tick_dir();
        self.pos = self
            .pos
            .neighbor(&self.curr_dir)
            .wrap((0, grid.rows()), (0, grid.cols()));

        // update came_from
        let x = CameFrom::Real((self.pos.get_u32(), prev_pos));
        self.came_from_head = Some(x);
    }

    fn move_body(&mut self, grid: &mut Grid) {
        if !self.spawn_timer.is_done() {
            grid.add(self.pos, Entity::ImmortalSnakeHead(self.id));
            let x = CameFrom::Real((self.pos.get_u32(), self.pos.get_u32()));
            self.came_from_head = Some(x);
            return;
        }

        // move body
        self.body.push_front(self.pos);
        grid.add(self.pos, Entity::SnakeBody(self.id));

        self.grow_count = match self.grow_count.take() {
            Some(x) => x.decr(1),

            None => {
                let last = self.body.pop_back().unwrap();
                grid.remove(last);

                if let Some(new_last) = self.body.back() {
                    let x = CameFrom::Real((new_last.get_u32(), last.get_u32()));
                    self.came_from_tail = Some(x);
                }

                // update dummy tail
                self.prev_last = last;

                None
            }
        }
    }

    fn grow(&mut self, n: NonZeroUsize) {
        self.grow_count = match self.grow_count.take() {
            None => Some(n),
            Some(m) => Some(NonZeroUsize::add(m, n)),
        };
    }

    fn kill(&mut self) {
        self.is_dead = true;
        //  1 + is for the dummy tail
        self.death_timer = Some(Timer::new(1 + self.body.len()));
    }

    fn remove(&mut self, grid: &mut Grid) {
        for idx in self.body.iter() {
            grid.remove(*idx);
        }

        self.body.clear();
    }

    fn give_direction(&mut self, dir: Direction) {
        if self.spawn_timer.is_done() {
            self.next_dir = Some(dir);
        } else {
            self.curr_dir = dir;
        }
    }

    fn tick_dir(&mut self) {
        self.next_dir.take().map(|next_dir| match self.curr_dir {
            Direction::Up | Direction::Down => match next_dir {
                Direction::Left | Direction::Right => {
                    self.curr_dir = next_dir;
                }
                _ => (),
            },

            Direction::Left | Direction::Right => match next_dir {
                Direction::Up | Direction::Down => {
                    self.curr_dir = next_dir;
                }
                _ => (),
            },
        });
    }
}

// +++++++++++++++
// + PropManager +
// +++++++++++++++

struct PropManager {
    props: Vec<(Option<Timer>, PropID, Index2D)>,
}

impl PropManager {
    fn new() -> Self {
        PropManager { props: Vec::new() }
    }

    fn add(&mut self, opt: Option<Timer>, pid: PropID, idx: Index2D) {
        self.props.push((opt, pid, idx))
    }

    fn tick(&mut self) {
        self.props.iter_mut().for_each(|(opt, _, _)| {
            if let Some(timer) = opt {
                timer.tick()
            }
        })
    }

    fn remove_by_id(&mut self, pid: PropID) {
        self.props.retain(|(_, pid2, _)| pid != *pid2)
    }

    fn indices_to_remove(&mut self) -> Vec<Index2D> {
        self.props
            .drain_filter(|(opt, _, _)| match opt {
                None => false,
                Some(timer) => timer.is_done(),
            })
            .map(|(_, _, idx)| idx)
            .collect()
    }
}

// +++++++++++++
// + GameState +
// +++++++++++++

pub struct GameState {
    grid: Grid,
    prop_manager: PropManager,
    // prop_spawner: Box<Fn() -> Box<Prop>>,
    prop_spawn_timer: Timer,
    snakes: HashMap<SnakeID, Snake>,
    prop_ids: std::ops::Range<u64>,
    snake_ids: std::ops::Range<u64>,
}

pub struct GameStateBuilder {
    rows: usize,
    cols: usize,
    // prop_spawner: Box<Fn() -> Box<Prop> + Send>,
    prop_spawn_timer: Timer,
}

impl GameStateBuilder {
    pub fn new() -> Self {
        GameStateBuilder {
            rows: 16,
            cols: 16,
            // prop_spawner: Box::new(food_spawner),
            prop_spawn_timer: Timer::new(5),
        }
    }

    pub fn with_prop_spawner(mut self, f: Box<Fn() -> Box<Prop> + Send>) -> Self {
        // self.prop_spawner = f;
        self
    }

    pub fn with_prop_spawn_timer(mut self, timer: Timer) -> Self {
        self.prop_spawn_timer = timer;
        self
    }

    pub fn with_dimensions(mut self, rows: usize, cols: usize) -> Self {
        self.rows = rows;
        self.cols = cols;
        self
    }

    pub fn build(self) -> GameState {
        GameState {
            grid: Grid::new(self.rows, self.cols),
            prop_manager: PropManager::new(),
            // prop_spawner: self.prop_spawner,
            prop_spawn_timer: self.prop_spawn_timer,
            snakes: HashMap::new(),
            prop_ids: std::ops::Range {
                start: 0,
                end: std::u64::MAX,
            },
            snake_ids: std::ops::Range {
                start: 0,
                end: std::u64::MAX,
            },
        }
    }
}

impl GameState {
    fn next_prop_id(&mut self) -> PropID {
        self.prop_ids.next().unwrap()
    }

    fn next_snake_id(&mut self) -> SnakeID {
        self.snake_ids.next().unwrap()
    }

    fn spawn_snake(grid: &Grid, id: SnakeID) -> Option<Snake> {
        grid.index_of_rand_vacant().map(|idx| Snake::new(id, idx))
    }

    fn snake_by_id(&mut self, id: SnakeID) -> &mut Snake {
        if let Some(sn) = self.snakes.get_mut(&id) {
            return sn;
        }

        panic!("error @ snake_by_id: unknown id {}", id)
    }

    fn tick_prop_spawn_timer(&mut self) {
        if self.prop_spawn_timer.is_done() {
            if let Some(pos) = self.grid.index_of_rand_vacant() {
                let pid = self.next_prop_id();
                // let prop = (self.prop_spawner)();
                let prop = food_spawner();
                self.prop_manager.add(prop.get_timer(), pid, pos);
                self.grid.add(pos, Entity::Prop(pid, prop));
            }

            self.prop_spawn_timer.reset();
        }

        self.prop_spawn_timer.tick();
    }

    fn tick_prop_manager(&mut self) {
        self.prop_manager.tick();
        let idcs = self.prop_manager.indices_to_remove();
        for idx in idcs {
            self.grid.remove(idx);
        }
    }

    fn tick_props(&mut self) {
        self.tick_prop_spawn_timer();
        self.tick_prop_manager();
    }

    pub fn tick(&mut self) {
        for sn in self.snakes.values_mut() {
            sn.tick(&mut self.grid);

            if !sn.is_dead {
                sn.remove_head(&mut self.grid);
                sn.move_body(&mut self.grid);
                sn.tick_head(&mut self.grid);
            }
        }

        let mut collisions: HashMap<(usize, usize), Vec<Entity>> =
            HashMap::new();

        for sn in self.snakes.values().filter(|sn| sn.is_mortal()) {
            let pos = sn.pos.get();
            let en = Entity::SnakeHead(sn.id);

            if collisions.contains_key(&pos) {
                collisions.get_mut(&pos).unwrap().push(en);
            } else {
                let mut ens = vec![en];
                let (i, j) = pos;

                self.grid.grid[i][j].take().map(|en2| ens.push(en2));
                collisions.insert(pos, ens);
            }
        }

        let mut prop_evs = Vec::new();
        let mut snake_evs = Vec::new();

        for ((i, j), ens) in collisions.drain() {
            let (en, prop_evs2, snake_evs2) = Entity::collide_many(ens);
            self.grid.grid[i][j] = en;
            prop_evs.extend(prop_evs2);
            snake_evs.extend(snake_evs2);
        }

        self.process_prop_events(prop_evs);
        self.process_snake_events(snake_evs);

        for snake in self.snakes.values_mut() {
            if snake.is_dead && snake.should_spawn() {
                snake.remove(&mut self.grid);
                GameState::spawn_snake(&self.grid, snake.id).map(|mut snake2| {
                    snake2.score = snake.score;
                    *snake = snake2;
                });
                snake.move_body(&mut self.grid);
            }
        }

        // tick_props() uses grid, so it needs to be called when grid is in
        // a consistant state. This is a good place to do it, just before
        // the draw.
        self.tick_props();
    }

    fn process_prop_events(&mut self, evs: Vec<PropEvent>) {
        for ev in evs {
            match ev {
                PropEvent::Remove(id) => self.prop_manager.remove_by_id(id),
            }
        }
    }

    fn process_snake_events(&mut self, evs: Vec<SnakeEvent>) {
        for ev in evs {
            let sn = self.snake_by_id(ev.id);

            match ev.cmd {
                Cmd::Kill => sn.kill(),
                Cmd::Grow(n) => sn.grow(n),
                Cmd::GiveScore(n) => sn.give_score(n),
                Cmd::GiveBuff(b) => sn.give_buff(b.get()),
            }
        }
    }

    pub fn builder() -> GameStateBuilder {
        GameStateBuilder::new()
    }

    pub fn add_snake(&mut self) -> Option<SnakeID> {
        let id = self.next_snake_id();
        GameState::spawn_snake(&self.grid, id).map(|sn| {
            self.snakes.insert(id, sn);
            id
        })
    }

    pub fn remove_snake(&mut self, id: SnakeID) -> Result<(), ()> {
        match self.snakes.remove(&id) {
            None => Err(()),
            Some(_) => Ok(()),
        }
    }

    pub fn get_score(&self, id: SnakeID) -> Option<u64> {
        self.snakes.get(&id).map(|sn| sn.score as u64)
    }

    pub fn give_direction(
        &mut self,
        id: SnakeID,
        dir: Direction,
    ) -> Result<(), ()> {
        match self.snakes.get_mut(&id) {
            Some(sn) => {
                sn.give_direction(dir);
                Ok(())
            }
            None => Err(()),
        }
    }

    pub fn get_grid_data(&self) -> GridData {
        self.grid.data()
    }

    pub fn get_game_data(&mut self) -> GameData {
        let mut came_from_heads: HashMap<SnakeID, CameFrom> = HashMap::new();
        let mut came_from_tails: HashMap<SnakeID, CameFrom> = HashMap::new();

        for sn in self.snakes.values_mut() {
            if let Some(x) = sn.came_from_head.take() {
                came_from_heads.insert(sn.id, x);
            }
            if let Some(x) = sn.came_from_tail.take() {
                came_from_tails.insert(sn.id, x);
            }
        }

        GameData {
            came_from_heads: came_from_heads,
            came_from_tails: came_from_tails,
            grid_data: self.get_grid_data(),
        }
    }
}
