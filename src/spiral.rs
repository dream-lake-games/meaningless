use bevy::prelude::*;

pub(crate) fn level_to_pos(level: usize) -> IVec2 {
    if level == 0 {
        return IVec2::ZERO;
    }

    let mut ring = 1usize;
    let mut start_of_ring = 1usize;

    loop {
        let ring_size = 4 * ring;
        if level < start_of_ring + ring_size {
            break;
        }
        start_of_ring += ring_size;
        ring += 1;
    }

    let pos_in_ring = level - start_of_ring;
    let r = ring as i32;
    let quadrant = pos_in_ring / ring;
    let offset = (pos_in_ring % ring) as i32;

    match quadrant {
        0 => IVec2::new(r - offset, -offset),
        1 => IVec2::new(-offset, -r + offset),
        2 => IVec2::new(-r + offset, offset),
        3 => IVec2::new(offset, r - offset),
        _ => unreachable!(),
    }
}

pub(crate) fn pos_to_level(pos: IVec2) -> Option<usize> {
    if pos == IVec2::ZERO {
        return Some(0);
    }

    let ring = (pos.x.abs() + pos.y.abs()) as usize;
    if ring == 0 {
        return Some(0);
    }

    let start_of_ring = if ring == 1 { 1 } else { 1 + 2 * (ring - 1) * ring };
    let r = ring as i32;

    let (quadrant, offset) = if pos.x > 0 && pos.y <= 0 {
        (0, (r - pos.x) as usize)
    } else if pos.x <= 0 && pos.y < 0 {
        (1, (-pos.x) as usize)
    } else if pos.x < 0 && pos.y >= 0 {
        (2, pos.y as usize)
    } else if pos.x >= 0 && pos.y > 0 {
        (3, pos.x as usize)
    } else {
        return None;
    };

    let level = start_of_ring + quadrant * ring + offset;
    if level_to_pos(level) == pos {
        Some(level)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spiral() {
        assert_eq!(level_to_pos(0), IVec2::new(0, 0));
        assert_eq!(level_to_pos(1), IVec2::new(1, 0));
        assert_eq!(level_to_pos(2), IVec2::new(0, -1));
        assert_eq!(level_to_pos(3), IVec2::new(-1, 0));
        assert_eq!(level_to_pos(4), IVec2::new(0, 1));
        assert_eq!(level_to_pos(5), IVec2::new(2, 0));
        assert_eq!(level_to_pos(6), IVec2::new(1, -1));
        assert_eq!(level_to_pos(7), IVec2::new(0, -2));
        assert_eq!(level_to_pos(8), IVec2::new(-1, -1));
    }
}
