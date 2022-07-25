use macroquad::{
    color::{colors, Color},
    input::{self, KeyCode},
    main,
    math::{Vec2, Vec4},
    rand, shapes, time,
    window::{self, Conf},
};
use std::cell::RefCell;

const ENTITY_COUNT: usize = 100;

fn config() -> Conf {
    Conf {
        window_title: "Balls".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[main(config)]
async fn main() {
    let mut app = App::new(
        colors::BLACK,
        Constraint::default(),
        Vec4::new(600.0, 300.0, 200.0, 200.0),
        Resolver::default(),
        25.0,
        colors::WHITE,
        ENTITY_COUNT,
    );

    app.run().await
}

struct App {
    background_color: Color,
    entities: Vec<RefCell<Entity>>,
    border: Constraint,
    resolver: Resolver,
}

impl App {
    pub fn new(
        background_color: Color,
        border: Constraint,
        spawn_area: Vec4,
        resolver: Resolver,
        entity_radius: f32,
        entity_color: Color,
        entity_count: usize,
    ) -> Self {
        let entities = Self::generate_entities(
            Vec2::new(spawn_area.x, spawn_area.y),
            Vec2::new(spawn_area.z, spawn_area.w),
            entity_radius,
            entity_color,
            entity_count,
        )
        .into_iter()
        .map(|entity| RefCell::new(entity))
        .collect();

        Self {
            background_color,
            entities,
            border,
            resolver,
        }
    }

    pub async fn run(&mut self) {
        // let mut ball = Entity::new(
        //     25.0,
        //     colors::WHITE,
        //     Motion::new(
        //         self.border.position.x - self.border.radius + 25.0,
        //         self.border.position.y,
        //     ),
        // );

        loop {
            if input::is_key_released(KeyCode::Escape) {
                break;
            };

            self.tick().await
        }
    }

    async fn tick(&mut self) {
        let dt = time::get_frame_time();

        window::clear_background(self.background_color);

        self.update(dt);
        self.draw();

        window::next_frame().await
    }

    fn update(&mut self, dt: f32) {
        self.resolver.update(&self.entities, &self.border, dt);
    }

    fn draw(&self) {
        self.border.draw();
        self.entities
            .iter()
            .for_each(|entity| entity.borrow().draw());
    }

    fn generate_entities(
        position: Vec2,
        dimensions: Vec2,
        radius: f32,
        color: Color,
        n: usize,
    ) -> Vec<Entity> {
        (0..n)
            .map(|_| {
                let x = rand::gen_range(position.x, position.x + dimensions.x);
                let y = rand::gen_range(position.y, position.y + dimensions.y);

                Entity::new(radius, color, Motion::new(x, y))
            })
            .collect()
    }
}

#[derive(Debug)]
struct Entity {
    radius: f32,
    color: Color,
    motion: Motion,
}

impl Entity {
    pub fn new(radius: f32, color: Color, motion: Motion) -> Self {
        Self {
            radius,
            color,
            motion,
        }
    }

    pub fn draw(&self) {
        shapes::draw_poly(
            self.motion.position.x,
            self.motion.position.y,
            100,
            self.radius,
            0.0,
            self.color,
        )
    }
}

#[derive(Debug)]
struct Motion {
    position: Vec2,
    previous_position: Vec2,
    acceleration: Vec2,
}

impl Motion {
    pub fn new(x: f32, y: f32) -> Self {
        let position = Vec2::new(x, y);

        Self {
            position,
            previous_position: position,
            acceleration: Vec2::new(0.0, 0.0),
        }
    }

    fn update_position(&mut self, dt: f32) {
        let velocity = self.position - self.previous_position;

        self.previous_position = self.position;
        self.position += self.acceleration + velocity * dt * dt;
        self.acceleration = Vec2::default();
    }

    pub fn accelerate(&mut self, acceleration: Vec2) {
        self.acceleration += acceleration;
    }
}

#[derive(Debug)]
struct Constraint {
    position: Vec2,
    radius: f32,
    offset: f32,
    color: Color,
}

impl Constraint {
    pub fn new(position: Vec2, radius: f32, offset: f32, color: Color) -> Self {
        Self {
            position,
            radius,
            offset,
            color,
        }
    }

    pub fn draw(&self) {
        shapes::draw_poly(
            self.position.x,
            self.position.y,
            100,
            self.radius,
            // self.radius + self.offset * 2.5,
            0.0,
            self.color,
        );
    }
}

impl Default for Constraint {
    fn default() -> Self {
        Self::new(Vec2::new(800.0, 450.0), 400.0, 25.0, colors::GRAY)
    }
}

#[derive(Debug)]
struct Resolver {
    gravity: Vec2,
}

impl Resolver {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            gravity: Vec2::new(x, y),
        }
    }

    fn update(&self, entities: &Vec<RefCell<Entity>>, constraint: &Constraint, dt: f32) {
        self.apply_gravity(entities);
        self.apply_constraint(entities, constraint);
        self.apply_collisions(entities);
        self.update_position(entities, dt);
    }

    fn update_position(&self, entities: &Vec<RefCell<Entity>>, dt: f32) {
        entities
            .iter()
            .map(|entity| entity.borrow_mut())
            .for_each(|mut entity| entity.motion.update_position(dt));
    }

    fn apply_gravity(&self, entities: &Vec<RefCell<Entity>>) {
        entities
            .iter()
            .map(|entity| entity.borrow_mut())
            .for_each(|mut entity| entity.motion.accelerate(self.gravity));
    }

    fn apply_constraint(&self, entities: &Vec<RefCell<Entity>>, constraint: &Constraint) {
        entities
            .iter()
            .map(|entity| entity.borrow_mut())
            .for_each(|mut entity| {
                let to_entity = entity.motion.position - constraint.position;
                let distance = to_entity.length();

                if distance > constraint.radius - constraint.offset {
                    let n = to_entity / distance;
                    entity.motion.position =
                        constraint.position + n * (distance - constraint.offset);
                }
            });
    }

    fn apply_collisions(&self, entities: &Vec<RefCell<Entity>>) {
        let entity_count = entities.len();
        let entity_offset = entities[0].borrow().radius * 2.0;

        for i in 0..entity_count {
            let mut entity_a = entities[i].borrow_mut();

            for k in i + 1..entity_count {
                let mut entity_b = entities[k].borrow_mut();
                let collision_axis = entity_a.motion.position - entity_b.motion.position;
                let distance = collision_axis.length();

                if distance < entity_offset {
                    let n = collision_axis / distance;
                    let delta = entity_offset - distance;

                    entity_a.motion.position += 0.5 * delta * n;
                    entity_b.motion.position -= 0.5 * delta * n;
                }
            }
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new(0.0, 10.0)
    }
}
