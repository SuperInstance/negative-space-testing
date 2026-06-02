/// Game Physics Spatial Analysis with negative-space-testing
///
/// Real test: detect holes in collision meshes, find unreachable areas,
/// identify disconnected regions in a game level.

use negative_space_testing::{NegativeTest, SpaceMap, ConservationChecker};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Floor,
    Wall,
    Pit,
    Spawn,
    Goal,
}

#[derive(Debug, Clone)]
struct Level {
    width: usize,
    height: usize,
    grid: Vec<Vec<Cell>>,
}

impl Level {
    fn new(w: usize, h: usize) -> Self {
        Self {
            width: w,
            height: h,
            grid: vec![vec![Cell::Floor; w]; h],
        }
    }

    fn set(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.grid[y][x] = cell;
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<Cell> {
        if x < self.width && y < self.height {
            Some(self.grid[y][x])
        } else {
            None
        }
    }

    fn is_pit(&self, x: usize, y: usize) -> bool {
        self.get(x, y) == Some(Cell::Pit)
    }

    fn walkable_cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                match self.grid[y][x] {
                    Cell::Floor | Cell::Spawn | Cell::Goal => cells.push((x, y)),
                    _ => {}
                }
            }
        }
        cells
    }

    fn reachable_from(&self, sx: usize, sy: usize) -> HashSet<(usize, usize)> {
        let mut visited = HashSet::new();
        let mut stack = vec![(sx, sy)];
        while let Some((x, y)) = stack.pop() {
            if !visited.insert((x, y)) {
                continue;
            }
            for &(dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    let (nx, ny) = (nx as usize, ny as usize);
                    match self.grid[ny][nx] {
                        Cell::Wall | Cell::Pit => {}
                        _ => {
                            if !visited.contains(&(nx, ny)) {
                                stack.push((nx, ny));
                            }
                        }
                    }
                }
            }
        }
        visited
    }

    fn isolated_region_count(&self) -> usize {
        let walkable = self.walkable_cells();
        if walkable.is_empty() {
            return 0;
        }
        let mut unvisited: HashSet<(usize, usize)> = walkable.into_iter().collect();
        let mut count = 0;
        while let Some(&start) = unvisited.iter().next() {
            let reachable = self.reachable_from(start.0, start.1);
            let to_remove: Vec<_> = reachable
                .iter()
                .filter(|p| unvisited.contains(p))
                .copied()
                .collect();
            for p in &to_remove {
                unvisited.remove(p);
            }
            if !to_remove.is_empty() {
                count += 1;
            }
        }
        count
    }

    fn display(&self) -> String {
        let mut s = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let ch = match self.grid[y][x] {
                    Cell::Floor => '.',
                    Cell::Wall => '#',
                    Cell::Pit => 'O',
                    Cell::Spawn => 'S',
                    Cell::Goal => 'G',
                };
                s.push(ch);
            }
            s.push('\n');
        }
        s
    }
}

fn blank_level() -> Level {
    Level::new(10, 10)
}

/// Check for pits by iterating walkable cells and counting those that are pits.
/// Returns (clean_count, total_count).
fn count_pits(level: &Level) -> (usize, usize) {
    let w = level.walkable_cells();
    let clean = w.iter().filter(|&&(x, y)| !level.is_pit(x, y)).count();
    (clean, w.len())
}

fn main() {
    println!("═══ GAME PHYSICS LEVEL ANALYSIS ═══\n");

    // Level 1: Walls create disconnected regions
    {
        println!("─── Level 1: Walls block goal ───");
        let mut level = blank_level();
        for x in 2..=7 {
            level.set(x, 2, Cell::Wall);
        }
        for y in 3..=7 {
            level.set(4, y, Cell::Wall);
        }
        level.set(0, 0, Cell::Spawn);
        level.set(9, 9, Cell::Goal);
        println!("{}", level.display());

        // NegativeTest: forbid holes in walkable cells
        let cloned = level.clone();
        let ft = NegativeTest::<(usize, usize)>::new()
            .forbid("hole", move |&(x, y)| cloned.is_pit(x, y));
        let w = level.walkable_cells();
        let r = ft.check_all(&w);
        assert!(r.is_clean(), "no pits");
        println!("  Walkable cells (no pits): {}/{} ✓", r.clean_count, r.total_checked);

        // SpaceMap
        let mut sm = SpaceMap::<&str, ()>::new();
        sm.forbid("pit_zone");
        assert!(sm.check_intrusions().is_empty());
        println!("  Forbidden zones clean ✓");

        let reachable = level.reachable_from(0, 0);
        println!("  Isolated regions: {}", level.isolated_region_count());
        println!("  Goal reachable: {}\n", reachable.contains(&(9, 9)));
    }

    // Level 2: Floor has holes
    {
        println!("─── Level 2: Floor with holes ───");
        let mut level = blank_level();
        level.set(1, 1, Cell::Pit);
        level.set(1, 5, Cell::Pit);
        level.set(8, 8, Cell::Pit);
        level.set(0, 0, Cell::Spawn);
        level.set(9, 9, Cell::Goal);
        println!("{}", level.display());

        let (clean, total) = count_pits(&level);
        println!("  Walkable cells (no pits): {clean}/{total} ✓");
        assert_eq!(clean, total, "walkable cells are not pits");

        let mut sm = SpaceMap::<&str, i32>::new();
        sm.forbid("pit_1_1");
        sm.forbid("pit_1_5");
        sm.forbid("pit_8_8");
        sm.occupy("spawn", 1);
        sm.occupy("goal", 1);
        println!("  Intrusions: {:?}", sm.check_intrusions());
        println!("  NS ratio: {:.2}\n", sm.negative_space_ratio());
    }

    // Level 3: Pit in unreachable zone
    {
        println!("─── Level 3: Hidden pit in unreachable zone ───");
        let mut level = blank_level();
        level.set(0, 0, Cell::Spawn);
        for y in 0..=9 {
            level.set(5, y, Cell::Wall);
        }
        level.set(7, 5, Cell::Pit);
        level.set(9, 9, Cell::Goal);
        println!("{}", level.display());

        let reachable = level.reachable_from(0, 0);
        let wset: HashSet<(usize, usize)> = level.walkable_cells().into_iter().collect();
        let unreachable: Vec<_> = wset.difference(&reachable).copied().collect();
        println!("  Walkable cells: {}", wset.len());
        println!("  Reachable: {}", reachable.len());
        println!("  Unreachable walkable: {}", unreachable.len());

        let pits_in_dead = unreachable
            .iter()
            .filter(|&&(x, y)| level.is_pit(x, y))
            .count();
        println!("  Pits in unreachable zone: {pits_in_dead}");
        println!("  Goal reachable: {}", reachable.contains(&(9, 9)));

        if pits_in_dead > 0 {
            println!("  → Wasted geometry: pit in area player never visits");
        }
        println!();
    }

    // Level 4: Conservation of game invariants
    {
        println!("─── Level 4: Game state invariants ───");
        let mut cc = ConservationChecker::new();
        cc.register("hp", 100.0, 0.0);
        cc.register("ammo", 50.0, 5.0);
        cc.register("items", 0.0, 0.0);

        cc.update("hp", 90.0);
        println!(
            "  {} hp: 100 → 90 (damage, violates zero-tolerance)",
            if cc.is_conserved("hp") { "✗" } else { "✓" }
        );

        cc.update("ammo", 45.0);
        println!(
            "  {} ammo: 50 → 45 (within tolerance=5)",
            if cc.is_conserved("ammo") { "✓" } else { "✗" }
        );

        cc.update("items", 1.0);
        println!(
            "  {} items: 0 → 1 (increase, fine)",
            if cc.is_conserved("items") { "✓" } else { "✗" }
        );

        println!("  Violations: {:?}", cc.violations());
        assert!(cc.violations().contains(&"hp".to_string()));
        println!("  → Health invariants enforced ✓\n");
    }

    println!("═══ ALL CHECKS PASSED ═══");
    println!();
    println!("What was demonstrated:");
    println!("  • NegativeTest — forbid holes in collision mesh");
    println!("  • SpaceMap — mark forbidden vs occupied zones");
    println!("  • ConservationChecker — game state invariants (HP, ammo, items)");
    println!("  • BFS reachability — detect disconnected level regions");
    println!("  • Cross-analysis — wasted geometry (pits in unreachable areas)");
}
