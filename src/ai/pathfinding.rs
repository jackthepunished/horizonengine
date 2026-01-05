//! A* pathfinding on a 2D grid
//!
//! Simple grid-based navigation for AI agents.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use glam::Vec2;
use rustc_hash::FxHashMap;

/// A 2D navigation grid
#[derive(Debug, Clone)]
pub struct Grid {
    /// Width in cells
    pub width: usize,
    /// Height in cells
    pub height: usize,
    /// Cell size in world units
    pub cell_size: f32,
    /// Walkable cells (true = walkable)
    cells: Vec<bool>,
    /// World origin offset
    pub origin: Vec2,
}

impl Grid {
    /// Create a new grid (all cells walkable by default)
    #[must_use]
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        Self {
            width,
            height,
            cell_size,
            cells: vec![true; width * height],
            origin: Vec2::ZERO,
        }
    }

    /// Set a cell's walkability
    pub fn set_walkable(&mut self, x: usize, y: usize, walkable: bool) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = walkable;
        }
    }

    /// Check if a cell is walkable
    #[must_use]
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.cells[y * self.width + x]
    }

    /// Convert world position to grid coordinates
    #[must_use]
    pub fn world_to_grid(&self, pos: Vec2) -> (i32, i32) {
        let local = pos - self.origin;
        (
            (local.x / self.cell_size).floor() as i32,
            (local.y / self.cell_size).floor() as i32,
        )
    }

    /// Convert grid coordinates to world position (center of cell)
    #[must_use]
    pub fn grid_to_world(&self, x: usize, y: usize) -> Vec2 {
        self.origin
            + Vec2::new(
                (x as f32 + 0.5) * self.cell_size,
                (y as f32 + 0.5) * self.cell_size,
            )
    }

    /// Get neighbors of a cell (4-directional)
    fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::with_capacity(4);

        if x > 0 && self.is_walkable(x - 1, y) {
            result.push((x - 1, y));
        }
        if x + 1 < self.width && self.is_walkable(x + 1, y) {
            result.push((x + 1, y));
        }
        if y > 0 && self.is_walkable(x, y - 1) {
            result.push((x, y - 1));
        }
        if y + 1 < self.height && self.is_walkable(x, y + 1) {
            result.push((x, y + 1));
        }

        result
    }
}

/// Result of pathfinding
#[derive(Debug, Clone)]
pub struct PathResult {
    /// Waypoints in world coordinates
    pub waypoints: Vec<Vec2>,
    /// Total path length
    pub length: f32,
}

impl PathResult {
    /// Check if path was found
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }
}

impl Default for PathResult {
    fn default() -> Self {
        Self {
            waypoints: Vec::new(),
            length: 0.0,
        }
    }
}

/// A* node for priority queue
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Node {
    x: usize,
    y: usize,
    g_cost: f32, // Cost from start
    f_cost: f32, // g_cost + heuristic
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other
            .f_cost
            .partial_cmp(&self.f_cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Find a path using A* algorithm
#[must_use]
pub fn find_path(grid: &Grid, start: Vec2, goal: Vec2) -> PathResult {
    let (start_x, start_y) = grid.world_to_grid(start);
    let (goal_x, goal_y) = grid.world_to_grid(goal);

    // Validate coordinates
    if start_x < 0 || start_y < 0 || goal_x < 0 || goal_y < 0 {
        return PathResult {
            waypoints: Vec::new(),
            length: 0.0,
        };
    }

    let start_x = start_x as usize;
    let start_y = start_y as usize;
    let goal_x = goal_x as usize;
    let goal_y = goal_y as usize;

    if !grid.is_walkable(start_x, start_y) || !grid.is_walkable(goal_x, goal_y) {
        return PathResult {
            waypoints: Vec::new(),
            length: 0.0,
        };
    }

    // A* implementation
    let mut open_set = BinaryHeap::new();
    let mut came_from: FxHashMap<(usize, usize), (usize, usize)> = FxHashMap::default();
    let mut g_score: FxHashMap<(usize, usize), f32> = FxHashMap::default();

    let heuristic = |x: usize, y: usize| -> f32 {
        let dx = (x as f32 - goal_x as f32).abs();
        let dy = (y as f32 - goal_y as f32).abs();
        dx + dy // Manhattan distance
    };

    g_score.insert((start_x, start_y), 0.0);
    open_set.push(Node {
        x: start_x,
        y: start_y,
        g_cost: 0.0,
        f_cost: heuristic(start_x, start_y),
    });

    while let Some(current) = open_set.pop() {
        if current.x == goal_x && current.y == goal_y {
            // Reconstruct path
            let mut path = vec![(goal_x, goal_y)];
            let mut curr = (goal_x, goal_y);

            while let Some(&prev) = came_from.get(&curr) {
                path.push(prev);
                curr = prev;
            }

            path.reverse();

            let waypoints: Vec<Vec2> = path
                .iter()
                .map(|&(x, y)| grid.grid_to_world(x, y))
                .collect();

            let length = calculate_path_length(&waypoints);

            return PathResult { waypoints, length };
        }

        for (nx, ny) in grid.neighbors(current.x, current.y) {
            let tentative_g = g_score.get(&(current.x, current.y)).unwrap_or(&f32::MAX) + 1.0;

            if tentative_g < *g_score.get(&(nx, ny)).unwrap_or(&f32::MAX) {
                came_from.insert((nx, ny), (current.x, current.y));
                g_score.insert((nx, ny), tentative_g);

                let f = tentative_g + heuristic(nx, ny);
                open_set.push(Node {
                    x: nx,
                    y: ny,
                    g_cost: tentative_g,
                    f_cost: f,
                });
            }
        }
    }

    // No path found
    PathResult {
        waypoints: Vec::new(),
        length: 0.0,
    }
}

/// Calculate total path length
fn calculate_path_length(waypoints: &[Vec2]) -> f32 {
    let mut length = 0.0;
    for i in 1..waypoints.len() {
        length += waypoints[i].distance(waypoints[i - 1]);
    }
    length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_pathfinding() {
        let mut grid = Grid::new(10, 10, 1.0);

        // Create a wall
        for y in 2..8 {
            grid.set_walkable(5, y, false);
        }

        let path = find_path(&grid, Vec2::new(2.5, 5.5), Vec2::new(8.5, 5.5));

        assert!(!path.is_empty());
        assert!(path.waypoints.len() > 2); // Should go around the wall
    }

    #[test]
    fn test_direct_path() {
        let grid = Grid::new(10, 10, 1.0);

        let path = find_path(&grid, Vec2::new(0.5, 0.5), Vec2::new(3.5, 0.5));

        assert!(!path.is_empty());
        assert_eq!(path.waypoints.len(), 4); // 4 cells in a line
    }

    #[test]
    fn test_no_path() {
        let mut grid = Grid::new(5, 5, 1.0);

        // Block everything around goal
        grid.set_walkable(3, 2, false);
        grid.set_walkable(3, 4, false);
        grid.set_walkable(2, 3, false);
        grid.set_walkable(4, 3, false);
        grid.set_walkable(3, 3, false);

        let path = find_path(&grid, Vec2::new(0.5, 0.5), Vec2::new(3.5, 3.5));

        assert!(path.is_empty());
    }
}
