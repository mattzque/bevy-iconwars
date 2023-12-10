use bevy::math::Vec2;
use bevy::utils::{EntityHashMap, EntityHashSet};
use std::fmt::Debug;
use std::hash::Hash;

/// Create Iterator over x and y (cartesian product) between -distance and +distance offset                                                                                                         
fn offset_iter(x: i16, y: i16, distance: i16) -> impl Iterator<Item = (i16, i16)> {
    let x_range = (x - distance)..=(x + distance);
    let y_range = (y - distance)..=(y + distance);
    x_range.flat_map(move |x| y_range.clone().map(move |y| (x, y)))
}

#[derive(Debug, Clone)]
pub struct SpatialQueryResult<T> {
    pub key: T,
    pub position: Vec2,
    pub velocity: Vec2,
    pub distance: f32,
}

#[derive(Debug, Clone)]
pub struct SpatialQuery2Result<'a, T> {
    pub key: T,
    pub position: &'a Vec2,
    pub distance: f32,
}

pub struct SpatialIndex<T> {
    pub min: Vec2,
    pub max: Vec2,
    pub grid_x: usize,
    pub grid_y: usize,
    pub len: usize,
    pub cell_size: f32,
    pub entities: Vec<EntityHashSet<T>>,
    pub by_entity: EntityHashMap<T, (usize, Vec2, Vec2)>,
}
impl<T> SpatialIndex<T>
where
    T: Hash + Eq + Clone + Debug,
{
    pub fn new(min: Vec2, max: Vec2, cell_size: f32) -> Self {
        let size = max - min;
        let grid_x = (size.x / cell_size).ceil() as usize;
        let grid_y = (size.y / cell_size).ceil() as usize;
        let len = grid_x * grid_y;
        Self {
            min,
            max,
            grid_x,
            grid_y,
            len,
            cell_size,
            entities: Vec::from_iter((0..len).map(|_| EntityHashSet::default())),
            by_entity: EntityHashMap::default(),
        }
    }

    fn pos_to_index(&self, position: Vec2) -> usize {
        let x = ((position.x - self.min.x) / self.cell_size) as usize;
        let y = ((position.y - self.min.y) / self.cell_size) as usize;
        self.grid_x * y + x
    }

    pub fn insert(&mut self, entity: T, position: Vec2, velocity: Vec2) {
        if let Some((old_index, old_position, old_velocity)) = self.by_entity.get(&entity) {
            if *old_position != position || *old_velocity != velocity {
                self.entities[*old_index].remove(&entity);
                self.by_entity.remove(&entity);
            }
        }

        let index = self.pos_to_index(position);
        if index > 0 && index < self.len {
            self.entities[index].insert(entity.clone());
            self.by_entity.insert(entity, (index, position, velocity));
        } else {
            self.by_entity.remove(&entity);
        }
    }

    pub fn simple_query(
        &self,
        position: Vec2,
        distance: f32,
    ) -> impl Iterator<Item = (&T, &Vec2, &Vec2)> + '_ {
        let grid_distance = (distance / self.cell_size).ceil() as i16;

        let x = ((position.x - self.min.x) / self.cell_size) as i16;
        let y = ((position.y - self.min.y) / self.cell_size) as i16;

        offset_iter(x, y, grid_distance)
            .flat_map(|(x, y)| {
                let index = self.grid_x as i16 * y + x;
                // println!("x:{} y:{} index={}", x, y, index);
                self.entities
                    .get(index as usize)
                    .into_iter()
                    .flat_map(|entities| entities.iter())
            })
            .map(move |entity| {
                let (_, other_position, other_velocity) = self.by_entity.get(entity).unwrap();
                (entity, other_position, other_velocity)
            })
    }

    pub fn query(
        &self,
        position: Vec2,
        distance: f32,
    ) -> impl Iterator<Item = SpatialQueryResult<T>> + '_ {
        let grid_distance = (distance / self.cell_size).ceil() as i16;

        let x = ((position.x - self.min.x) / self.cell_size) as i16;
        let y = ((position.y - self.min.y) / self.cell_size) as i16;

        offset_iter(x, y, grid_distance)
            .flat_map(|(x, y)| {
                let index = self.grid_x as i16 * y + x;
                self.entities
                    .get(index as usize)
                    .into_iter()
                    .flat_map(|entities| entities.iter())
            })
            .flat_map(move |entity| {
                let (_, other_position, other_velocity) = self.by_entity.get(entity).unwrap();
                let other_position = *other_position;
                let distance_to_other = (position - other_position).length();
                if distance_to_other <= distance {
                    Some(SpatialQueryResult {
                        key: entity.clone(),
                        position: other_position,
                        velocity: *other_velocity,
                        distance: distance_to_other,
                    })
                } else {
                    None
                }
            })
    }
}
