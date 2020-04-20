#[cfg(debug_assertions)]
extern crate console_error_panic_hook;
extern crate hashers;
extern crate nalgebra as na;
extern crate poisson;
extern crate rand;
extern crate wee_alloc;

use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::BuildHasherDefault;

use na::{Point2, Vector2};
use noise::{NoiseFn, Perlin, Seedable};
use poisson::{algorithm, Builder, Type};
use rand::distributions::Normal;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use wasm_bindgen::prelude::*;

use engine::key::{key_codes, KeyManager};
use engine::renderer::{Renderer, Texture, TextureMap};
use engine::{Collider, GameObject, Rend, World};

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct TexturedBox {
    size: na::Vector2<f32>,
    texture: engine::renderer::Texture,
}

impl Rend for TexturedBox {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject) {
        renderer.draw_quad(game_object.pos, self.size, &self.texture)
    }
}

struct Thermometer {
    size: na::Vector2<f32>,
    body_pos: (f32, f32),
    filling_pos: (f32, f32),
    texture_size: (f32, f32),
    texture_map: TextureMap,
    temperature: f32,
}

impl Thermometer {
    pub fn new(
        body_pos: (f32, f32),
        filling_pos: (f32, f32),
        texture_size: (f32, f32),
        texture_map: TextureMap,
    ) -> Thermometer {
        Thermometer {
            size: na::Vector2::new(128.0, 32.0),
            body_pos,
            filling_pos,
            texture_size,
            texture_map,
            temperature: 0.5,
        }
    }
}

impl Rend for Thermometer {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject) {
        renderer.draw_quad_with_depth(
            game_object.pos,
            self.size,
            &self.texture_map.get_texture_custom(
                self.body_pos.0,
                self.body_pos.1,
                self.texture_size.0,
                self.texture_size.1,
            ),
            -0.11,
        );
        let mut filling_size = self.size.clone_owned();
        filling_size.x *= self.temperature;
        let mut filling_pos = game_object.pos.clone();
        filling_pos.x -= self.size.x * 0.5 * (1.0 - self.temperature);
        renderer.draw_quad_with_depth(
            filling_pos,
            filling_size,
            &self.texture_map.get_texture_custom(
                self.filling_pos.0,
                self.filling_pos.1,
                self.texture_size.0 * self.temperature,
                self.texture_size.1,
            ),
            -0.11,
        );
    }
}

struct Inventory {
    size: na::Vector2<f32>,
    texture_pos: (f32, f32),
    texture_size: (f32, f32),
    texture_map: TextureMap,
    amount: u8,
}

impl Inventory {
    pub fn new(
        texture_pos: (f32, f32),
        texture_size: (f32, f32),
        texture_map: TextureMap,
    ) -> Inventory {
        Inventory {
            size: na::Vector2::new(40.0, 40.0),
            texture_pos,
            texture_size,
            texture_map,
            amount: 0,
        }
    }
}

impl Rend for Inventory {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject) {
        let mut size = self.size.clone_owned();
        size.y *= self.amount as f32;
        renderer.draw_quad_with_depth(
            game_object.pos,
            size,
            &self.texture_map.get_texture_custom(
                self.texture_pos.0,
                self.texture_pos.1,
                self.texture_size.0,
                self.texture_size.1 * self.amount as f32,
            ),
            -0.11,
        );
    }
}

struct Grass {
    texture_map: TextureMap,
}

impl Grass {
    pub fn new(texture_map: TextureMap) -> Grass {
        Grass { texture_map }
    }
}
impl Rend for Grass {
    fn render(&self, renderer: &mut Renderer, _game_object: &GameObject) {
        let cam = renderer.get_camera();
        let vp = renderer.get_viewport();
        let pos = na::Point2::new(cam.x, cam.y - vp.y / 2.0);
        let size = vp;
        renderer.draw_quad_with_depth(
            pos,
            size,
            &self.texture_map.get_very_custom(
                na::Vector2::new(cam.x / 1024.0, -cam.y / 1024.0),
                vp / 1024.0,
            ),
            -pos.y - vp.y,
        );
    }
}

struct Cam {}

impl Rend for Cam {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject) {
        renderer.set_camera(game_object.pos);
    }
}

struct Fire {}

impl Rend for Fire {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject) {
        renderer.set_fire_heat(1.0);
        renderer.set_fire_pos(game_object.pos);
    }
}

struct SomeWorld {
    game_objects: HashMap<String, GameObject, BuildHasherDefault<hashers::fnv::FNV1aHasher32>>,
    last_tick: f64,
}

const WORLD_EDGE: f64 = 10000.0;

impl SomeWorld {
    fn new() -> SomeWorld {
        let spritesheet = TextureMap::new(4, 4, "spritesheet".to_string());

        let mut player = GameObject::new(na::Point2::new(350.0, 250.0));
        player.add_rend(Box::new(TexturedBox {
            size: na::Vector2::new(128.0, 128.0),
            texture: spritesheet.get_texture(1, 0),
        }));
        player.add_rend(Box::new(Cam {}));
        let mut fire = GameObject::new(na::Point2::new(300.0, 280.0));
        fire.add_collider(Collider::new(32.0));
        fire.add_rend(Box::new(TexturedBox {
            size: na::Vector2::new(64.0, 64.0),
            texture: spritesheet.get_texture(2, 0),
        }));
        fire.add_rend(Box::new(Fire {}));

        let mut thermometer = GameObject::new(na::Point2::new(0.0, 150.0));
        thermometer.add_rend(Box::new(Thermometer::new(
            (2.0, 0.0),
            (2.0, 0.5),
            (2.0, 0.5),
            engine::renderer::TextureMap::new(4, 1, "ui".to_string()),
        )));

        let mut inventory = GameObject::new(na::Point2::new(-50.0, 0.0));
        inventory.add_rend(Box::new(Inventory::new(
            (0.0, 0.0),
            (1.0, 1.0),
            engine::renderer::TextureMap::new(4, 1, "ui".to_string()),
        )));

        let mut grass = GameObject::new(na::Point2::new(0.0, 0.0));
        grass.add_rend(Box::new(Grass::new(engine::renderer::TextureMap::new(
            1,
            1,
            "grass".to_string(),
        ))));

        let mut game_objects =
            HashMap::with_hasher(BuildHasherDefault::<hashers::fnv::FNV1aHasher32>::default());
        game_objects.insert("player".to_string(), player);
        game_objects.insert("fire".to_string(), fire);
        game_objects.insert("thermometer".to_string(), thermometer);
        game_objects.insert("inventory".to_string(), inventory);
        game_objects.insert("grass".to_string(), grass);

        // let perlin = Perlin::new().set_seed(2);
        const TREE_COLLISION_RANGE: f32 = 16.0;
        // const PERLIN_SCALER: f64 = 20.0; // smaller number = bigger features
        // let mut treshold = SmallRng::seed_from_u64(0);
        let mut tree_i = 0;
        // for sample in Builder::<_, na::Vector2<f64>>::with_radius(
        //     TREE_COLLISION_RANGE as f64 * 2.0 / WORLD_EDGE,
        //     Type::Normal,
        // )
        // .build(SmallRng::seed_from_u64(0), algorithm::Bridson)
        // .generate()
        // {
        //     let perlin_coords: [f64; 2] =
        //         (*(&sample * PERLIN_SCALER).as_slice()).try_into().unwrap();
        //     // if (perlin.get(perlin_coords)) > treshold.gen_range(0.0, 1.5) {
        //     if (perlin.get(perlin_coords)) > treshold.sample(Normal::new(0.0, 0.5)) {
        // let world_coords = (&sample - Vector2::new(0.5, 0.5)) * WORLD_EDGE;
        // log::debug!("{:?}", world_coords);
        for (x, y) in trees::TREES.iter() {
            let mut tree =
                // GameObject::new(Point2::new(world_coords.x as f32, world_coords.y as f32));
                GameObject::new(Point2::new(*x, *y));
            tree.add_collider(Collider::new(TREE_COLLISION_RANGE));
            tree.add_rend(Box::new(TexturedBox {
                size: na::Vector2::new(128.0, 128.0),
                texture: spritesheet.get_texture(1, 0),
            }));
            tree.props.insert("tree".to_string(), tree_i as f32);
            game_objects.insert(format!("tree{}", tree_i), tree);
            tree_i += 1;
        }
        // }
        log::debug!("Got trees: {}", tree_i);
        SomeWorld {
            game_objects,
            last_tick: 0.0,
        }
    }

    fn get_direction(key_manager: &KeyManager) -> na::Vector2<f32> {
        let mut direction = na::Vector2::zeros();
        if key_manager.key_pressed(key_codes::W) || key_manager.key_pressed(key_codes::UP_ARROW) {
            direction.y += 1.0;
        }
        if key_manager.key_pressed(key_codes::S) || key_manager.key_pressed(key_codes::DOWN_ARROW) {
            direction.y += -1.0;
        }
        if key_manager.key_pressed(key_codes::D) || key_manager.key_pressed(key_codes::RIGHT_ARROW)
        {
            direction.x += 1.0;
        }
        if key_manager.key_pressed(key_codes::A) || key_manager.key_pressed(key_codes::LEFT_ARROW) {
            direction.x += -1.0;
        }
        direction
    }
}

impl engine::World for SomeWorld {
    fn tick(&mut self, key_manager: &KeyManager, timestamp: f64) {
        let direction = SomeWorld::get_direction(key_manager);

        let player = self.game_objects.get("player").unwrap();

        let mut speed = player.speed.clone();
        speed += direction * ((timestamp - self.last_tick) as f32 * 0.1);
        let norm = speed.norm();
        if norm > 10.0 {
            speed *= 10.0 / norm;
        }
        speed *= 0.8;

        if norm * 0.8 < 1.0 {
            speed *= 0.0;
        }

        let player_pos = player.pos.clone();
        let mut stumps = HashMap::new();
        let mut inventory = self.game_objects.get_mut("inventory").unwrap().rend[0]
            .downcast_mut::<Inventory>()
            .unwrap()
            .amount;
        self.game_objects.retain(|key, game_object| {
            if let Some(collider) = game_object.get_collider() {
                if collider.collide(&game_object, &player_pos, &mut speed) {
                    if key_manager.key_up(key_codes::E) && game_object.props.contains_key("tree") {
                        // Chopping trees creates a stump and a log
                        let mut stump = GameObject::new(game_object.pos.clone());
                        stump.add_collider(Collider::new(5.0));
                        stump.add_rend(Box::new(TexturedBox {
                            size: na::Vector2::new(128.0, 128.0),
                            texture: TextureMap::new(4, 4, "spritesheet".to_string())
                                .get_texture(3, 0),
                        }));
                        stump
                            .props
                            .insert("stump".to_string(), *game_object.props.get("tree").unwrap());
                        stumps.insert(format!("stump{}", key), stump);
                        let mut log = GameObject::new(&game_object.pos + Vector2::new(32.0, 32.0));
                        log.add_collider(Collider::new(10.0));
                        log.add_rend(Box::new(TexturedBox {
                            size: na::Vector2::new(64.0, 64.0),
                            texture: TextureMap::new(4, 4, "spritesheet".to_string())
                                .get_texture(0, 2),
                        }));
                        log.props
                            .insert("log".to_string(), *game_object.props.get("tree").unwrap());
                        stumps.insert(format!("log{}", key), log);
                        return false;
                    } else if key_manager.key_up(key_codes::SPACE)
                        && game_object.props.contains_key("log")
                        && inventory < 3
                    {
                        // Picking up a log
                        inventory += 1;
                        return false;
                    }
                }
            }
            true
        });
        self.game_objects.extend(stumps);
        // Dropping off logs:
        let fire = self.game_objects.get("fire").unwrap();
        if fire
            .get_collider()
            .as_ref()
            .unwrap()
            .collide(&fire, &player_pos, &mut speed)
            && key_manager.key_up(key_codes::SPACE)
        {
            inventory -= 1;
        }
        self.game_objects.get_mut("inventory").unwrap().rend[0]
            .downcast_mut::<Inventory>()
            .unwrap()
            .amount = inventory;

        let player = self.game_objects.get("player").unwrap();
        let fire = self.game_objects.get("fire").unwrap();
        let conductivity = (timestamp - self.last_tick) as f32 / 8000.0;
        let c = 300.0; // smaller number == sharper drop-off
        let r2 = ((f32::max(0.0, (player.pos - fire.pos).norm() - 48.0) + c) / c).powi(2);
        self.game_objects.get_mut("thermometer").unwrap().rend[0]
            .downcast_mut::<Thermometer>()
            .unwrap()
            .temperature *= 1.0 - conductivity;
        self.game_objects.get_mut("thermometer").unwrap().rend[0]
            .downcast_mut::<Thermometer>()
            .unwrap()
            .temperature += 1.0 / r2 * conductivity;

        let mut player = self.game_objects.get_mut("player").unwrap();
        player.pos += speed * (timestamp - self.last_tick) as f32 * 0.05;
        player.speed = speed;
        self.last_tick = timestamp;
    }

    fn get_game_objects(&self) -> Vec<&GameObject> {
        self.game_objects.values().collect()
    }
}

#[wasm_bindgen]
pub fn run() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Game starting");
    engine::start(Box::new(SomeWorld::new()) as Box<dyn World>).unwrap();
}

mod trees;
