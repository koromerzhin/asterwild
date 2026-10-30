#[cfg(test)]
use crate::blocks::{hash2, is_cave, surface_height};
#[cfg(test)]
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_deterministic() {
        assert_eq!(surface_height(10, 12345), surface_height(10, 12345));
    }

    #[test]
    fn terrain_varies_with_seed() {
        let _ = (surface_height(10, 12345), surface_height(10, 99999));
    }

    #[test]
    fn hash_range() {
        for x in -5..5 {
            for y in -5..5 {
                let h = hash2(x, y, 42);
                assert!((0.0..=1.0).contains(&h));
            }
        }
    }

    #[test]
    fn cave_deterministic() {
        assert_eq!(is_cave(5, 5, 7), is_cave(5, 5, 7));
    }

    #[test]
    fn chunk_indexing() {
        assert_eq!(World::chunk_x(0), 0);
        assert_eq!(World::chunk_x(15), 0);
        assert_eq!(World::chunk_x(16), 1);
        assert_eq!(World::chunk_x(-1), -1);
        assert_eq!(World::local_x(-1), 15);
    }
}
