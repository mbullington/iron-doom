use ultraviolet::Vec2;

use crate::helpers::geom::Polygon2d;

pub fn point_in_polygon(point: &Vec2, polygon: &Polygon2d) -> bool {
    let mut k = 0;

    let x = point.x;
    let y = point.y;

    // We only have 1 contour, polygon.points.
    let contour_len = polygon.points.len();
    let contour = &polygon.points;

    let mut u1 = contour[0].x - x;
    let mut v1 = contour[0].y - y;

    for ii in 0..contour_len {
        let next_p = {
            if ii + 1 == contour_len {
                contour[0]
            } else {
                contour[ii + 1]
            }
        };

        let v2 = next_p.y - y;

        if (v1 < 0.0 && v2 < 0.0) || (v1 > 0.0 && v2 > 0.0) {
            v1 = v2;
            u1 = next_p.x - x;
            continue;
        }

        let u2 = next_p.x - point.x;

        if v2 > 0.0 && v1 <= 0.0 {
            let f = (u1 * v2) - (u2 * v1);
            if f > 0.0 {
                k += 1;
            } else if f == 0.0 {
                return false;
            }
        } else if v1 > 0.0 && v2 <= 0.0 {
            let f = (u1 * v2) - (u2 * v1);
            if f < 0.0 {
                k += 1;
            } else if f == 0.0 {
                return false;
            }
        } else if (v2 == 0.0 && v1 < 0.0) || (v1 == 0.0 && v2 < 0.0) {
            let f = u1 * v2 - u2 * v1;
            if f == 0.0 {
                return false;
            }
        } else if v1 == 0.0 && v2 == 0.0 && ((u2 <= 0.0 && u1 >= 0.0) || (u1 <= 0.0 && u2 >= 0.0)) {
            return false;
        }
        v1 = v2;
        u1 = u2;
    }

    if k % 2 == 0 {
        return false;
    }

    true
}
