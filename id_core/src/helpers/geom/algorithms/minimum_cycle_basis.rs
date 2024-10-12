use multimap::MultiMap;
use ultraviolet::Vec2;

use crate::helpers::geom::{Graph2d, GraphVertIndex, Winding};

// Extensions for ultraviolet.

pub trait _Vec2Ext {
    fn perp_dot(&self, other: Self) -> f32;
}

impl _Vec2Ext for Vec2 {
    fn perp_dot(&self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }
}

fn _remove_filament_at(
    v: GraphVertIndex,
    gravestones: &mut [bool],
    adj_map: &mut MultiMap<GraphVertIndex, GraphVertIndex>,
) {
    let mut current: Option<GraphVertIndex> = Some(v);
    loop {
        if current.is_none() {
            break;
        }

        let current_idx = current.unwrap();
        let current_adj = adj_map.get_vec(&current_idx).unwrap();

        if current_adj.len() >= 2 {
            break;
        }

        // Mark gravestone.
        gravestones[current_idx] = true;

        if !current_adj.is_empty() {
            let next = current_adj[0];
            _remove_edge(current_idx, next, adj_map);
            current = Some(next);
        } else {
            current = None;
        }
    }
}

fn _remove_edge(
    v1: GraphVertIndex,
    v2: GraphVertIndex,
    adj_map: &mut MultiMap<GraphVertIndex, GraphVertIndex>,
) {
    {
        let adj = adj_map.get_vec_mut(&v1).unwrap();
        adj.retain(|v| *v != v2);
    }
    {
        let adj = adj_map.get_vec_mut(&v2).unwrap();
        adj.retain(|v| *v != v1);
    }
}

fn _reduce_walk(mut walk: Vec<GraphVertIndex>) -> Vec<GraphVertIndex> {
    for i in 1..walk.len() {
        if i >= walk.len() {
            break;
        }
        let idup = walk.iter().rposition(|&v| v == walk[i]).unwrap();
        if idup > i {
            walk.drain(i + 1..idup);
        }
    }

    walk
}

fn _closed_walk_from(
    v: GraphVertIndex,
    vertices: &[Vec2],
    adj_map: &MultiMap<GraphVertIndex, GraphVertIndex>,
) -> Vec<GraphVertIndex> {
    let mut walk: Vec<GraphVertIndex> = Vec::new();
    let mut curr = v;
    let mut prev: Option<GraphVertIndex> = None;

    loop {
        // If the walk contains a loop, we're done.
        if walk.contains(&curr) && curr != v {
            walk.clear();
            break;
        }

        walk.push(curr);
        (curr, prev) = {
            let val = _get_next(curr, prev, vertices, adj_map);
            (val.0, Some(val.1))
        };

        if curr == v {
            break;
        }
    }

    walk
}

fn _get_next(
    v: GraphVertIndex,
    v_prev: Option<GraphVertIndex>,
    vertices: &[Vec2],
    adj_map: &MultiMap<GraphVertIndex, GraphVertIndex>,
) -> (GraphVertIndex, GraphVertIndex) {
    let adj = adj_map.get_vec(&v).unwrap();
    if adj.len() == 1 {
        (adj[0], v)
    } else {
        let next = _best_by_winding(
            v_prev,
            v,
            match v_prev {
                Some(_) => Winding::CounterClockwise,
                None => Winding::Clockwise,
            },
            vertices,
            adj_map,
        );
        (next, v)
    }
}

fn _best_by_winding(
    v_prev: Option<GraphVertIndex>,
    v_curr: GraphVertIndex,
    winding: Winding,
    vertices: &[Vec2],
    adj_map: &MultiMap<GraphVertIndex, GraphVertIndex>,
) -> GraphVertIndex {
    let d_curr = if let Some(v_prev) = v_prev {
        vertices[v_curr] - vertices[v_prev]
    } else {
        Vec2::new(0.0, -1.0)
    };

    let mut adj = adj_map.get_vec(&v_curr).unwrap().clone();
    if let Some(v_prev) = v_prev {
        adj.retain(|v| *v != v_prev);
    }

    adj.into_iter().fold(v_curr, |v_so_far, v| {
        if v_so_far == v_curr {
            return v;
        }

        let v_is_better = _better_by_winding(
            &vertices[v],
            &vertices[v_so_far],
            &vertices[v_curr],
            &d_curr,
            winding,
        );

        if v_is_better {
            v
        } else {
            v_so_far
        }
    })
}

fn _better_by_winding(
    v: &Vec2,
    v_so_far: &Vec2,
    v_curr: &Vec2,
    d_curr: &Vec2,
    winding: Winding,
) -> bool {
    let d = *v - *v_curr;
    let d_so_far = *v_so_far - *v_curr;

    let is_convex = d_so_far.dot(*d_curr) > 0.0;
    let curr2v = d_curr.perp_dot(d);
    let vsf2v = d_so_far.perp_dot(d);

    match winding {
        Winding::Clockwise => {
            (is_convex && (curr2v >= 0.0 || vsf2v >= 0.0))
                || (!is_convex && curr2v >= 0.0 && vsf2v >= 0.0)
        }
        Winding::CounterClockwise => {
            (!is_convex && (curr2v < 0.0 || vsf2v < 0.0))
                || (is_convex && curr2v < 0.0 && vsf2v < 0.0)
        }
    }
}

/// Given an undirected weighted graph, find the Minimum Cycle Basis.
///
/// This is a greedy algorithm that will find elementary cycles, i.e cycles
/// that do not contain a vertex more than once.
///
/// It is based on this library (ISC License, check README at root of project):
/// https://github.com/vbichkovsky/min-cycles
pub fn minimum_cycle_basis(graph: &Graph2d) -> Vec<Vec<GraphVertIndex>> {
    let mut cycles: Vec<Vec<GraphVertIndex>> = Vec::new();
    if graph.vertices.len() < 3 {
        return cycles;
    }

    let mut adj_map: MultiMap<GraphVertIndex, GraphVertIndex> = graph.vert_adj_map.clone();

    let vertices: &Vec<Vec2> = &graph.vertices;
    let mut gravestones: Vec<bool> = vec![false; graph.vertices.len()];

    loop {
        // Find vertex in the bottom-left.
        let v = vertices
            .iter()
            .enumerate()
            .filter(|(i, _)| !gravestones[*i])
            .fold(0, |v_so_far, (i, v)| {
                if gravestones[v_so_far] {
                    return i;
                }

                if v.x < vertices[v_so_far].x
                    || (v.x == vertices[v_so_far].x && v.y < vertices[v_so_far].y)
                {
                    i
                } else {
                    v_so_far
                }
            });

        // If we've selected a gravestone, we're done.
        if v == 0 && gravestones[v] {
            break;
        }

        let walk = _reduce_walk(_closed_walk_from(v, vertices, &adj_map));
        if walk.len() == 0 {
            gravestones[v] = true;
            continue;
        }

        // Remove the edge from the walk.
        _remove_edge(walk[0], walk[1], &mut adj_map);

        _remove_filament_at(walk[0], &mut gravestones, &mut adj_map);
        _remove_filament_at(walk[1], &mut gravestones, &mut adj_map);

        if walk.len() > 2 {
            cycles.push(walk)
        }
    }

    cycles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_example() {
        let line_graph = Graph2d::new(
            vec![
                // outer box
                Vec2::new(-10., -10.),
                Vec2::new(-10., 10.),
                Vec2::new(10., -10.),
                Vec2::new(10., 10.),
                // inner box
                Vec2::new(-5., -5.),
                Vec2::new(-5., 5.),
                Vec2::new(5., -5.),
                Vec2::new(5., 5.),
            ],
            // create two boxes, i.e. a polygon with a hole
            vec![
                // outer box
                (0, 1),
                (1, 3),
                (3, 2),
                (2, 0),
                // inner box
                (4, 5),
                (5, 7),
                (7, 6),
                (6, 4),
                // add a little connective tissue for fun
                (0, 4),
            ],
        );

        let cycles = minimum_cycle_basis(&line_graph);

        assert_eq!(cycles.len(), 2);
        assert_eq!(cycles[0], vec![0, 1, 3, 2]);
        assert_eq!(cycles[1], vec![4, 5, 7, 6]);
    }

    #[test]
    fn ambigious_case() {
        let line_graph = Graph2d::new(
            vec![
                // box
                Vec2::new(-10., -10.),
                Vec2::new(-10., 10.),
                Vec2::new(10., -10.),
                Vec2::new(10., 10.),
            ],
            // create two boxes, i.e. a polygon with a hole
            vec![
                // box
                (0, 1),
                (1, 3),
                (3, 2),
                (2, 0),
                // connector down the middle
                (0, 3),
            ],
        );

        let cycles = minimum_cycle_basis(&line_graph);

        assert_eq!(cycles.len(), 2);
        assert_eq!(cycles[0], vec![0, 1, 3]);
        assert_eq!(cycles[1], vec![0, 3, 2]);
    }
}
