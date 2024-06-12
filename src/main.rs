use bevy::asset::io::memory::Value::Vec;
use bevy::prelude::*;
use bevy::math::bounding::{
    Aabb2d,
    BoundingVolume,
    IntersectsVolume
};


const PLAYER_SPEED: f32 = 5.;
const PLAYER_ACCEL: f32 = 0.07;
const PLAYER_DECEL: f32 = 0.1;
const PLAYER_JUMP_STRENGTH:f32 = 8.;
const GRAVITY:f32 = -0.2;

const CAMERA_ACCEL: f32 = 0.1;


pub struct SpawnPlugin;
pub struct UpdatePlugin;
impl Plugin for SpawnPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Startup,(
            spawn_camera,
            spawn_player,
            spawn_block));
    }
}

impl Plugin for UpdatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,((control_player,
                                 gravitate,
                                 move_bodies,
                                 handle_collisions,
                                 camera_follow
                                ).chain(),
                                project_transforms
        ).chain());
    }
}

fn main() {
    App::new().add_plugins((DefaultPlugins, SpawnPlugin, UpdatePlugin)).run();
}

#[derive(Component)]
struct Position(Vec2);
#[derive(Component)]
struct Shape(Vec2);
#[derive(Component)]
struct VisShape(Vec2);
#[derive(Component)]
struct Velocity(Vec2);
#[derive(Component)]
struct Gravity(Vec2);

#[derive(Copy, Clone)]
enum Collision{
    Top,
    Bottom,
    Left,
    Right
}

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Block;
#[derive(Component)]
struct Camera;
#[derive(Component)]
struct Gravitated;
#[derive(Component)]
struct Collider;

#[derive(Bundle)]
struct BlockBundle{
    block: Block,
    shape: Shape,
    position: Position,
    collider: Collider
}

impl BlockBundle {
    fn new(position:Vec2,shape: Vec2) ->Self{
        Self{
            block:Block,
            shape:Shape(shape),
            position:Position(position),
            collider:Collider
        }
    }
}
#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    position: Position,
    shape: Shape,
    vis_shape: VisShape,
    gravity: Gravity,
    velocity: Velocity,
    gravitated: Gravitated,
}
impl PlayerBundle{
    fn new(position: Vec2, shape: Vec2) -> Self{
        Self{
            player: Player,
            gravitated: Gravitated,
            position: Position(position),
            shape: Shape(shape),
            vis_shape: VisShape(shape),
            gravity: Gravity(Vec2::new(0.,GRAVITY)),
            velocity: Velocity(Vec2::new(0.,2.)),
        }
    }
}
fn spawn_camera(
    mut commands: Commands
){
    commands.spawn((Camera2dBundle::default(),
                    Position(Vec2::new(0.,0.)),
                    Velocity(Vec2::new(0.,0.)),
                    Camera
    ));
}
fn spawn_player(
    mut commands:Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
){
    let shape = Vec2::new(60.,100.);
    commands.spawn((PlayerBundle::new(Vec2::new(0.0,0.),shape),
    ColorMesh2dBundle{
        mesh: meshes.add(Rectangle::new(shape.x,shape.y)).into(),
        material: materials.add(Color::WHITE),
        ..default()
    }));
}
fn spawn_block(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
){
    let shape = Vec2::new(500.,50.);
    let position = Vec2::new(0., -200.);
    commands.spawn((BlockBundle::new(position,shape),
    ColorMesh2dBundle{
        mesh: meshes.add(Rectangle::new(shape.x,shape.y)).into(),
        material: materials.add(Color::oklab(0.8,0.0,0.0)),
        ..default()
    }));
}

fn collide(
    body1: Aabb2d,
    body2: Aabb2d
) -> Option<(Collision,Vec2)>{
    if !body1.intersects(&body2){
        return None;
    }
    let closest_point = body2.closest_point(body1.center());
    let offset = body1.center() - closest_point;
    let mut clip_amount = offset;
    let side = if offset.x.abs() + body1.half_size().y > offset.y.abs() +body1.half_size().x {
        if offset.x > 0. {
            clip_amount = body1.half_size()- offset;
            Collision::Left
        } else {
            clip_amount = body1.half_size()+ offset;
            Collision::Right
        }
    } else if offset.y < 0. {
        clip_amount = body1.half_size()+ offset;
        Collision::Top
    } else {
        clip_amount = body1.half_size()- offset;
        Collision::Bottom
    };
    // println!("clip amount: {}",clip_amount);
    return Some((side, clip_amount))
}

fn gravitate(
    mut body: Query<(&mut Velocity, &Gravity), With<Gravitated>>
){
    for (mut velocity, gravity) in &mut body{
        velocity.0 += gravity.0
    }
}

fn handle_collisions(
    mut player: Query<(&mut Position, &mut Velocity, &Shape), With<Player>>,
    colliders: Query<(&Position,&Shape),(With<Collider>, Without<Player>)>
){
    for (position, shape) in & colliders{
        for (mut p_position, mut p_velocity, p_shape) in player.get_single_mut(){
            let p_aabb = Aabb2d::new(p_position.0, p_shape.0/2.0);
            let aabb = Aabb2d::new(position.0, shape.0/2.0);
            let collision = collide(p_aabb,aabb);
            if !collision.is_none(){
                let (collision, offset) = collision.unwrap();
                match collision{
                    Collision::Top => {
                        p_velocity.0.y = 0.;
                        p_position.0.y -= offset.y;
                    },
                    Collision::Bottom => {
                        p_velocity.0.y = 0.;
                        p_position.0.y += offset.y;
                    },
                    Collision::Left=> {
                        p_velocity.0.x = 0.;
                        p_position.0.x += offset.x;
                    },
                    Collision::Right => {
                        p_velocity.0.x = 0.;
                        p_position.0.x -= offset.x;
                    },
                }
            }
        }
    }
}

fn control_player(
    mut player: Query<(&mut Velocity), With<Player>>,
    kb_input: Res<ButtonInput<KeyCode>>
){
    if let Ok(mut velocity) = player.get_single_mut(){
        let mut target_x_speed = 0.;
        if kb_input.pressed(KeyCode::KeyD){
            target_x_speed = PLAYER_SPEED;
        }
        if kb_input.pressed(KeyCode::KeyA){
            target_x_speed = -PLAYER_SPEED;
        }
        if kb_input.just_pressed(KeyCode::KeyW) || kb_input.just_pressed(KeyCode::Space){
            velocity.0.y = PLAYER_JUMP_STRENGTH
        }

        if target_x_speed.abs() < velocity.0.x.abs(){
            velocity.0.x = flerp(velocity.0.x, target_x_speed, PLAYER_DECEL)
        }
        else{
            velocity.0.x = flerp(velocity.0.x, target_x_speed, PLAYER_ACCEL)
        }
    }
}
fn move_bodies(
    mut body: Query<(&mut Position, &Velocity)>
){
    for (mut position, velocity) in &mut body{
        position.0 += velocity.0
    }
}

fn camera_follow(
    mut camera_query: Query<(&mut Velocity, &Position), With<Camera>>,
    player_query: Query<&Position, With<Player>>,
){
    if let Ok(player_pos) = player_query.get_single() {
        for (mut camera_vel, camera_pos) in camera_query.iter_mut() {
            // Calculate the direction vector from the camera to the player
            let direction = player_pos.0 - camera_pos.0;

            // Calculate the distance to the player
            let distance = direction.length();

            // If the distance is significant, update the camera's velocity
            if distance > 0.1 {
                // Adjust the damping factor to control the "weight" feel
                let damping = 0.04;

                // Calculate the new velocity with damping
                let new_velocity = direction * damping;

                // Update the camera's velocity
                camera_vel.0 = camera_vel.0.lerp(new_velocity, 0.1);
            } else {
                // If the distance is small, stop the camera
                camera_vel.0 = Vec2::ZERO;
            }
        }
    }
}

fn project_transforms(
    mut positionables: Query<(&mut Transform, &Position)>
){
    for (mut transform,position) in &mut positionables{
        transform.translation = position.0.extend(0.)
    }
}
fn flerp(a:f32,b:f32,t:f32) -> f32{
    a + t * (b - a)
}
fn vlerp(a:Vec2, b:Vec2, t:f32)->Vec2{
    Vec2::new(
        flerp(a.x,b.x,t),
        flerp(a.y,b.y,t),
    )
}