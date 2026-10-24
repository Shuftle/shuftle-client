//! Laying UI out one way on portrait screens and another on landscape ones, as
//! UI has no media queries.

use bevy::prelude::*;

use crate::game::layout::TableLayout;

pub struct OrientPlugin;

impl Plugin for OrientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            orient_all.run_if(resource_changed::<TableLayout>),
        )
        .add_observer(orient_added);
    }
}

/// Lays a UI node's children out one way on portrait screens and another on
/// landscape ones.
#[derive(Component, Clone, Default)]
pub struct Orient {
    pub portrait: FlexDirection,
    pub landscape: FlexDirection,
}

impl Orient {
    fn apply(&self, layout: &TableLayout, node: &mut Mut<Node>) {
        let direction = if layout.is_portrait() {
            self.portrait
        } else {
            self.landscape
        };
        if node.flex_direction != direction {
            node.flex_direction = direction;
        }
    }
}

/// Orients a node as it's spawned.
fn orient_added(
    add: On<Add<Orient>>,
    layout: Res<TableLayout>,
    mut nodes: Query<(&Orient, &mut Node)>,
) {
    if let Ok((orient, mut node)) = nodes.get_mut(add.entity) {
        orient.apply(&layout, &mut node);
    }
}

/// Orients every node again once the screen turned.
fn orient_all(layout: Res<TableLayout>, mut nodes: Query<(&Orient, &mut Node)>) {
    for (orient, mut node) in &mut nodes {
        orient.apply(&layout, &mut node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spawns an oriented node from a scene, on a screen of `size`, and
    /// returns its direction.
    fn spawned_direction(size: Vec2) -> FlexDirection {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
        ))
        .insert_resource(TableLayout::new(size, 0.))
        .add_observer(orient_added);
        let world = app.world_mut();
        let node = world
            .spawn_scene(bsn! {
                Node
                Orient { portrait: FlexDirection::Column, landscape: FlexDirection::Row }
            })
            .unwrap()
            .id();
        world.get::<Node>(node).unwrap().flex_direction
    }

    #[test]
    fn scenes_spawn_oriented() {
        assert_eq!(
            spawned_direction(Vec2::new(411., 923.)),
            FlexDirection::Column
        );
        assert_eq!(spawned_direction(Vec2::new(923., 411.)), FlexDirection::Row);
    }
}
