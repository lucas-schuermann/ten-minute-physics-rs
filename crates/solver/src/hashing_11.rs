use glam::Vec3;

pub struct Hash {
    inv_spacing: f32,
    table_size: usize,
    cell_start: Vec<usize>,
    cell_entries: Vec<usize>,
    pub query_ids: Vec<usize>,
    pub query_size: usize,

    // for `query_all`
    max_num_objects: usize,
    pub first_adj_id: Vec<usize>,
    pub adj_ids: Vec<usize>,
}

impl Hash {
    #[must_use]
    pub fn new(spacing: f32, max_num_objects: usize) -> Self {
        let table_size = 2 * max_num_objects; // LVSTODO move to param
        Self {
            inv_spacing: 1.0 / spacing,
            table_size,
            cell_start: vec![0; table_size + 1],
            cell_entries: vec![0; max_num_objects],
            query_ids: vec![0; max_num_objects],
            query_size: 0,

            // for `query_all`
            max_num_objects,
            first_adj_id: vec![0; max_num_objects + 1],
            adj_ids: Vec::with_capacity(10 * max_num_objects), // LVSTODO move to const
        }
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    fn hash_coords(&self, xi: i32, yi: i32, zi: i32) -> usize {
        (i32::abs(
            xi.wrapping_mul(92_837_111_i32)
                ^ yi.wrapping_mul(689_287_499_i32)
                ^ zi.wrapping_mul(283_923_481_i32),
        ) % (self.table_size as i32)) as usize // fantasy function
    }

    fn int_coord(&self, coord: f32) -> i32 {
        f32::floor(coord * self.inv_spacing) as i32
    }

    fn hash_pos(&self, pos: &Vec3) -> usize {
        self.hash_coords(
            self.int_coord(pos.x),
            self.int_coord(pos.y),
            self.int_coord(pos.z),
        )
    }

    pub fn create(&mut self, positions: &[Vec3]) {
        // compute cell sizes
        self.cell_start.fill(0);
        self.cell_entries.fill(0);
        for &p in positions {
            let h = self.hash_pos(&p);
            self.cell_start[h] += 1;
        }

        // compute cell starts
        let mut start = 0;
        for i in 0..self.table_size {
            start += self.cell_start[i];
            self.cell_start[i] = start;
        }
        self.cell_start[self.table_size] = start; // guard

        // fill in object ids
        for (i, pos) in positions.iter().enumerate().take(self.cell_entries.len()) {
            let h = self.hash_pos(pos);
            self.cell_start[h] -= 1;
            self.cell_entries[self.cell_start[h]] = i;
        }
    }

    pub fn query(&mut self, pos: &Vec3, max_dist: f32) {
        let x0 = self.int_coord(pos.x - max_dist);
        let y0 = self.int_coord(pos.y - max_dist);
        let z0 = self.int_coord(pos.z - max_dist);

        let x1 = self.int_coord(pos.x + max_dist);
        let y1 = self.int_coord(pos.y + max_dist);
        let z1 = self.int_coord(pos.z + max_dist);

        self.query_size = 0;

        for xi in x0..=x1 {
            for yi in y0..=y1 {
                for zi in z0..=z1 {
                    let h = self.hash_coords(xi, yi, zi);
                    let start = self.cell_start[h];
                    let end = self.cell_start[h + 1];

                    for i in start..end {
                        self.query_ids[self.query_size] = self.cell_entries[i];
                        self.query_size += 1;
                    }
                }
            }
        }
    }

    // for use in `self_collision_15.rs`
    pub fn query_all(&mut self, positions: &[Vec3], max_dist: f32) {
        let max_dist_sq = max_dist * max_dist;
        self.adj_ids.clear();
        for i in 0..self.max_num_objects {
            let id0 = i;
            self.first_adj_id[id0] = self.adj_ids.len();
            self.query(&positions[id0], max_dist);

            for j in 0..self.query_size {
                let id1 = self.query_ids[j];
                if id1 >= id0 {
                    continue;
                }
                let dist_sq = positions[id0].distance_squared(positions[id1]);
                if dist_sq > max_dist_sq {
                    continue;
                }
                self.adj_ids.push(id1);
            }
        }
        self.first_adj_id[self.max_num_objects] = self.adj_ids.len();
    }
}
