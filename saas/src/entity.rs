use std::fmt;
use std::num::NonZeroUsize;

use crate::util::*;

extern crate serde;
use serde::{Deserialize, Serialize};

pub type ID = u64;
pub type SnakeID = u64;
pub type PropID = u64;

pub struct SnakeData {}

pub trait Buff {
    fn get_timer(&self) -> Timer;
    fn apply(&self, sd: SnakeData) -> SnakeData;
    fn id(&self) -> ID;
}

pub struct BuffPtr(Box<Buff>);

impl BuffPtr {
    pub fn new(b: Box<Buff>) -> Self {
        BuffPtr(b)
    }

    pub fn get(self) -> Box<Buff> {
        self.0
    }
}

impl PartialEq for BuffPtr {
    fn eq(&self, other: &BuffPtr) -> bool {
        self.0.id() == other.0.id()
    }
}

impl Eq for BuffPtr {}

impl fmt::Debug for BuffPtr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BuffPtr(id: {})", self.0.id())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Cmd {
    Kill,
    GiveScore(i32),
    GiveBuff(BuffPtr),
    Grow(NonZeroUsize),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SnakeEvent {
    pub id: SnakeID,
    pub cmd: Cmd,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PropEvent {
    Remove(PropID),
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Kind {
    None,
    Prop,
    SnakeHead,
    SnakeBody,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Tag {
    pub kind: Kind,
    pub id: ID,
}

pub enum CollisionResult {
    RemoveSelf,
    RemoveBoth,
    RemoveOther,
}

pub trait Prop {
    fn collision_result(&self) -> CollisionResult;
    fn collision_events(&self, id: SnakeID) -> Vec<SnakeEvent>;
    fn get_timer(&self) -> Option<Timer>;
    fn id(&self) -> ID;
}

pub enum Entity {
    Prop(PropID, Box<Prop>),
    SnakeBody(SnakeID),
    SnakeHead(SnakeID),
    ImmortalSnakeHead(SnakeID),
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Entity::Prop(_, p) => write!(f, "Entity::Prop(id: {:?})", p.id()),
            Entity::SnakeBody(id) => write!(f, "Entity::SnakeBody(id: {})", id),
            Entity::SnakeHead(id) => write!(f, "Entity::SnakeHead(id: {})", id),

            Entity::ImmortalSnakeHead(id) => {
                write!(f, "Entity::ImmmortalSnakeHead(id: {})", id)
            }
        }
    }
}

impl Entity {
    pub fn is_prop(&self) -> bool {
        match self {
            Entity::Prop(_, _) => true,
            _ => false,
        }
    }

    pub fn is_snake_head(&self) -> bool {
        match self {
            Entity::SnakeHead(_) => true,
            _ => false,
        }
    }

    fn collide(
        a: Entity,
        b: Entity,
    ) -> (Option<Entity>, Vec<PropEvent>, Vec<SnakeEvent>) {
        let (a, b) = if a.is_snake_head() { (a, b) } else { (b, a) };

        let mut prop_evs = Vec::new();
        let mut snake_evs = Vec::new();
        let mut out;

        let id = match a {
            Entity::SnakeHead(id) => id,
            _ => panic!("error @ collide: {:?}; {:?}", a, b),
        };

        match b {
            Entity::Prop(pid, p) => {
                snake_evs.extend(p.collision_events(id));

                let result = p.collision_result();
                match result {
                    CollisionResult::RemoveSelf => {
                        prop_evs.push(PropEvent::Remove(pid));
                        out = Some(a);
                    }

                    CollisionResult::RemoveBoth => {
                        prop_evs.push(PropEvent::Remove(pid));
                        out = None;
                    }
                    CollisionResult::RemoveOther => {
                        out = Some(Entity::Prop(pid, p));
                    }
                }
            }

            Entity::SnakeBody(_) | Entity::ImmortalSnakeHead(_) => {
                snake_evs.push(SnakeEvent {
                    id: id,
                    cmd: Cmd::Kill,
                });
                out = Some(b);
            }

            Entity::SnakeHead(id2) => {
                snake_evs.push(SnakeEvent {
                    id: id,
                    cmd: Cmd::Kill,
                });
                snake_evs.push(SnakeEvent {
                    id: id2,
                    cmd: Cmd::Kill,
                });
                out = None;
            }
        }

        (out, prop_evs, snake_evs)
    }

    fn collide_snake_heads(ens: Vec<Entity>) -> Vec<SnakeEvent> {
        let mut out = Vec::new();

        for en in ens.iter() {
            match en {
                Entity::SnakeHead(id) => out.push(SnakeEvent {
                    id: *id,
                    cmd: Cmd::Kill,
                }),

                _ => panic!("error @ collide_snake_heads: {:?}", en),
            }
        }

        out
    }

    pub fn collide_many(
        mut ens: Vec<Entity>,
    ) -> (Option<Entity>, Vec<PropEvent>, Vec<SnakeEvent>) {
        let n = ens.len();

        if n == 0 {
            return (None, Vec::new(), Vec::new());
        } else if n == 1 {
            return (Some(ens.swap_remove(0)), Vec::new(), Vec::new());
        } else if n == 2 {
            return Entity::collide(ens.swap_remove(0), ens.swap_remove(0));
        }

        let heads: Vec<Entity> =
            ens.drain_filter(|x| x.is_snake_head()).collect();

        let mut ens: Vec<Entity> =
            ens.into_iter().filter(|x| !x.is_snake_head()).collect();

        let n = ens.len();

        if n == 0 {
            let snake_evs = Entity::collide_snake_heads(heads);
            (None, Vec::new(), snake_evs)
        } else if n == 1 {
            match ens.swap_remove(0) {
                x @ Entity::SnakeBody(_) => {
                    let snake_evs = Entity::collide_snake_heads(heads);
                    (Some(x), Vec::new(), snake_evs)
                }

                x @ Entity::ImmortalSnakeHead(_) => {
                    let snake_evs = Entity::collide_snake_heads(heads);
                    (Some(x), Vec::new(), snake_evs)
                }

                Entity::Prop(pid, p) => {
                    let snake_evs = Entity::collide_snake_heads(heads);
                    let mut prop_evs = Vec::new();
                    let out = match p.collision_result() {
                        CollisionResult::RemoveSelf
                        | CollisionResult::RemoveBoth => {
                            prop_evs.push(PropEvent::Remove(pid));
                            None
                        }
                        CollisionResult::RemoveOther => {
                            Some(Entity::Prop(pid, p))
                        }
                    };

                    (out, prop_evs, snake_evs)
                }

                x => panic!("error @ collide_many: x = {:?}", x),
            }
        } else {
            let mut temp: Vec<Entity> = Vec::new();
            temp.extend(ens);
            temp.extend(heads);

            panic!("error @ collide_many: {:?}", temp);
        }
    }
}

impl Entity {
    pub fn tag(&self) -> Tag {
        match self {
            Entity::Prop(_, p) => Tag {
                kind: Kind::Prop,
                id: p.id(),
            },

            Entity::SnakeBody(id) => Tag {
                kind: Kind::SnakeBody,
                id: *id,
            },

            Entity::SnakeHead(id) => Tag {
                kind: Kind::SnakeHead,
                id: *id,
            },

            Entity::ImmortalSnakeHead(id) => Tag {
                kind: Kind::SnakeHead,
                id: *id,
            },
        }
    }
}
