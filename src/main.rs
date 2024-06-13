use bevy:: prelude::*;
use bevy_xpbd_3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_xpbd3d::*;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        PhysicsPlugins::default(),
        PhysicsDebugPlugin::default(),
        TnuaControllerPlugin::default(),
        TnuaXpbd3dPlugin::default(),
    ))
    .add_systems(Startup, (spawn_world,spawn_player,spawn_sensors))
    .add_systems(Update, (player_movement,sensor_detection))
    .run();
}
#[derive(Component)]
struct Player;

#[derive(Component)]
struct SensorArea;

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 26.0, 40.0)
            .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 8.0, 0.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::default().looking_at(-Vec3::Y, Vec3::Z),
        ..Default::default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(64.0, 64.0)),
            material: materials.add(Color::WHITE),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::halfspace(Vec3::Y),
    ));
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // player
    commands.spawn((
        Player,
        PbrBundle {
            mesh: meshes.add(Cuboid::new(2.5, 5.0, 2.5)),
            material: materials.add(Color::RED),
            ..default()
            },
        RigidBody::Dynamic,
        Collider::capsule(2.5, 1.5),
        TnuaControllerBundle::default(),
        TnuaXpbd3dSensorShape(Collider::cylinder(0.0, 0.49)),
        LockedAxes::ROTATION_LOCKED,
    ));
}

fn spawn_sensors(
    mut commands: Commands
) {
    let sensor_area1 = SpatialBundle { 
        transform: Transform { 
            translation: Vec3::new(-11.,4.5,0.), 
            ..default() }, 
        ..default() 
    };

    let sensor_area2 = SpatialBundle { 
        transform: Transform { 
            translation: Vec3::new(11.,4.5,0.), 
            ..default() }, 
        ..default() 
    };

    for sensor_area in [sensor_area1,sensor_area2] {
        commands.spawn((
            Sensor,
            SensorArea,
            Collider::cuboid(10.,8.,20.),
            sensor_area
        ))
        .with_children(|parent| {
            parent.spawn(PointLightBundle {
                point_light: PointLight {color: Color::RED, intensity: 4200000., ..default()},
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            });
        })
        ;
    };
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>, 
    mut query: Query<&mut TnuaController>
) {
    let Ok(mut controller) = query.get_single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) {
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction += Vec3::X;
    }

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * 20.0,
        float_height: 2.9,
        ..Default::default()
    });
}

fn sensor_detection(
    mut collision_event_reader: EventReader<CollisionStarted>,
    mut collision_event_reader2: EventReader<CollisionEnded>,
    player_q: Query<&Player>,
    mut lights_q: Query <&mut PointLight>,
    sensor_area_q: Query<(&SensorArea, &Children)>,
) {
    for CollisionStarted(entity1, entity2) in collision_event_reader.read() {
        for (entity1, entity2) in [(entity1, entity2), (entity2, entity1)] {
            if let (Ok(_player), Ok((_sensor_area,children))) = (player_q.get(*entity1), sensor_area_q.get(*entity2)) {
                println!(">ENTERED sensor id: {entity2:?} (player id: {entity1:?})");

                for child in children.iter() {
                    let Ok(mut light) = lights_q.get_mut(*child) else {
                        continue
                    };
                    light.color = Color::GREEN;
                }
            }
        }
    }

    for CollisionEnded(entity11, entity22) in collision_event_reader2.read() {
        for (entity11, entity22) in [(entity11, entity22), (entity22, entity11)] {
            if let (Ok(_player2), Ok((_sensor_area2, children2))) = (player_q.get(*entity11), sensor_area_q.get(*entity22)) {
                println!("<EXITED sensor id: {entity22:?} (player id: {entity11:?})");

                for child2 in children2.iter() {
                    let Ok(mut light2) = lights_q.get_mut(*child2) else {
                        continue
                    };
                    light2.color = Color::RED;
                }
            }
        }
    }
}