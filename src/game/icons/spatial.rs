use bevy::math::Vec2;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

type GridKey = (i16, i16);

pub struct SpatialQueryResult<'a, T> {
    pub key: T,
    pub position: &'a Vec2,
    pub velocity: &'a Vec2,
    pub distance: f32,
}

pub struct SpatialIndex<T> {
    pub min: Vec2,
    pub max: Vec2,
    pub cell_size: f32,
    pub entities: HashMap<GridKey, HashSet<T>>,
    pub by_entity: HashMap<T, (GridKey, Vec2, Vec2)>,
}
impl<T> SpatialIndex<T>
where
    T: Hash + Eq + Clone + Debug,
{
    // iterator: impl Iterator<Item = (Vec2, Entity)>
    pub fn new(min: Vec2, max: Vec2, cell_size: f32) -> Self {
        Self {
            min,
            max,
            cell_size,
            entities: HashMap::new(),
            by_entity: HashMap::new(),
        }
    }

    fn key(&self, position: Vec2) -> GridKey {
        let x = position.x.div_euclid(self.cell_size) as i16;
        let y = position.y.div_euclid(self.cell_size) as i16;
        (x, y)
    }

    pub fn insert(&mut self, entity: T, position: Vec2, velocity: Vec2) {
        let key = self.key(position);
        if let Some((old_key, old_position, _)) = self.by_entity.get(&entity) {
            if old_key == &key && *old_position == position {
                return;
            }
            self.entities.entry(*old_key).or_default().remove(&entity);
        }

        self.entities.entry(key).or_default().insert(entity.clone());
        self.by_entity.insert(entity, (key, position, velocity));
    }

    /// Returns (T, position, velocity, distance)
    pub fn query(
        &self,
        position: Vec2,
        distance: f32,
    ) -> impl Iterator<Item = SpatialQueryResult<'_, T>> + '_ {
        let grid_distance = (distance / self.cell_size).ceil() as i16;
        let key = self.key(position);
        let mut results = Vec::new();
        for x in (key.0 - grid_distance)..=(key.0 + grid_distance) {
            for y in (key.1 - grid_distance)..=(key.1 + grid_distance) {
                if let Some(entities) = self.entities.get(&(x, y)) {
                    entities.iter().for_each(|entity| results.push(entity));
                }
            }
        }

        results.into_iter().flat_map(move |entity| {
            let (_, other_position, other_velocity) = self.by_entity.get(entity).unwrap();
            let distance_to_other = (position - *other_position).length();
            if distance_to_other > 0.0 && distance_to_other <= distance {
                Some(SpatialQueryResult {
                    key: entity.clone(),
                    position: other_position,
                    velocity: other_velocity,
                    distance: distance_to_other,
                })
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::SpatialIndex;

    #[test]
    fn test_key() {
        let index =
            SpatialIndex::<u64>::new(Vec2::new(-100.0, 100.0), Vec2::new(-100.0, 100.0), 10.0);
        assert_eq!(index.key(Vec2::splat(0.0)), (0, 0));
        assert_eq!(index.key(Vec2::splat(5.0)), (0, 0));
        assert_eq!(index.key(Vec2::splat(-5.0)), (-1, -1));
        assert_eq!(index.key(Vec2::splat(10.0)), (1, 1));
        assert_eq!(index.key(Vec2::splat(20.0)), (2, 2));
        assert_eq!(index.key(Vec2::splat(25.0)), (2, 2));
        assert_eq!(index.key(Vec2::splat(-10.0)), (-1, -1));
        assert_eq!(index.key(Vec2::splat(-20.0)), (-2, -2));
        assert_eq!(index.key(Vec2::splat(-25.0)), (-3, -3));
    }

    #[test]
    fn test_spatial() {
        let mut index = SpatialIndex::new(Vec2::new(-100.0, 100.0), Vec2::new(-100.0, 100.0), 10.0);
        index.insert(1, Vec2::new(0.0, 0.0), Vec2::splat(0.0));
        index.insert(2, Vec2::new(-20.0, -20.0), Vec2::splat(0.0));
        index.insert(3, Vec2::new(20.0, 20.0), Vec2::splat(0.0));
        assert_eq!(
            index
                .query(Vec2::new(0.5, 0.5), 10.0)
                .map(|r| r.key)
                .collect::<Vec<_>>(),
            &[1]
        );
        let mut results = index
            .query(Vec2::new(0.5, 0.5), 50.0)
            .map(|r| r.key)
            .collect::<Vec<_>>();
        results.sort();
        assert_eq!(results, &[1, 2, 3]);
        let mut results = index
            .query(Vec2::new(20.0, 20.0), 30.0)
            .map(|r| r.key)
            .collect::<Vec<_>>();
        results.sort();
        assert_eq!(results, &[1, 3]);
    }
}
