# `bevy_newtonian2d`

Simple Newtonian Physics simulator for Bevy game engine.

Forces are accumulated in `PhysicsSystem::AccumulateForces` system and applied in `PhysicsSystem::ApplyForces` once per frame.
