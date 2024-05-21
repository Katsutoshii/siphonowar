use std::f32::consts::PI;

use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        experimental::taa::TemporalAntiAliasBundle,
        tonemapping::Tonemapping,
    },
    render::view::RenderLayers,
};

use crate::prelude::*;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(Msaa::Sample4)
            .add_event::<CameraMoveEvent>()
            .add_systems(Startup, startup)
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 1000.,
            });
    }
}

/// Used to help identify our main camera
pub fn startup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            projection: PerspectiveProjection {
                fov: MainCamera::FOV,
                near: 0.1,
                far: 2000.,
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(
                0.0,
                -zindex::CAMERA * MainCamera::THETA.tan(),
                zindex::CAMERA,
            )
            .with_rotation(Quat::from_axis_angle(Vec3::X, MainCamera::THETA)),
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings {
            intensity: 0.15 * 2.0,
            low_frequency_boost: 0.7,
            low_frequency_boost_curvature: 0.95,
            high_pass_frequency: 1.0,
            prefilter_settings: BloomPrefilterSettings {
                threshold: 0.0,
                threshold_softness: 0.0,
            },
            composite_mode: BloomCompositeMode::EnergyConserving,
        },
        CameraController::default(),
        InheritedVisibility::default(),
        RenderLayers::from_layers(&[0, 1]),
        // Add the setting to the camera.
        // This component is also used to determine on which camera to run the post processing effect.
        PostProcessSettings { intensity: 0.02 },
        TemporalAntiAliasBundle::default(),
        MainCamera,
    ));
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, zindex::CAMERA)
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, -PI / 6.)),
        directional_light: DirectionalLight {
            color: Color::ANTIQUE_WHITE,
            illuminance: 4500.,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}
