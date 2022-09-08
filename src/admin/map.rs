use std::path::PathBuf;

use bevy::prelude::*;
use bevy_egui::*;
use maps::TileMap;
use networking::{
    messaging::{AppExt, MessageEvent, MessageSender},
    NetworkManager,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct ChangeMapMessage {
    name: String,
}

fn client_map_selection_ui(mut egui_context: ResMut<EguiContext>, mut sender: MessageSender) {
    egui::Window::new("Load map").show(egui_context.ctx_mut(), |ui| {
        for &map_name in ["DeltaStation2", "BoxStation", "MetaStation"].iter() {
            if ui.button(map_name).clicked() {
                sender.send_to_server(&ChangeMapMessage {
                    name: map_name.to_owned(),
                });
            }
        }
    });
}

fn map_loader_system(
    mut messages: EventReader<MessageEvent<ChangeMapMessage>>,
    mut commands: Commands,
    server: Res<AssetServer>,
    tilemaps: Query<Entity, With<TileMap>>,
) {
    let message = match messages.iter().last() {
        Some(m) => &m.message,
        None => return,
    };

    // Delete existing maps
    for entity in tilemaps.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Add new map to load
    // Note: you better not send me any path traversal >:/
    let handle = server.load(PathBuf::from(format!("maps/{}.dmm", message.name)));
    commands.insert_resource(crate::Map {
        handle,
        spawned: false,
    });
}

pub struct MapManagementPlugin;

impl Plugin for MapManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_network_message::<ChangeMapMessage>();

        if app
            .world
            .get_resource::<NetworkManager>()
            .unwrap()
            .is_server()
        {
            app.add_system(map_loader_system);
        } else {
            app.add_system(client_map_selection_ui);
        }
    }
}
