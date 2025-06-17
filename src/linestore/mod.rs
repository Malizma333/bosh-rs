pub mod grid;
mod raw_store;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::game::Line;
    use crate::game::Vector2D;
    use crate::linestore::grid::Grid;

    const DEFAULT_CELL_SIZE: f64 = 14.0;

    #[test]
    fn infinite_slope_line() {
        let line = &Line::builder().point(0.0, 0.0).point(0.0, 100.0).build();

        let grid = Grid::new(vec![*line], DEFAULT_CELL_SIZE);

        let nearby = grid.lines_near(Vector2D(0.0, 0.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(0.0, 50.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(0.0, -10.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(25.0, 110.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(0.0, 150.0), 1);
        assert_eq!(nearby, Vec::<&Line>::new());

        let nearby = grid.lines_near(Vector2D(0.0, -25.0), 1);
        assert_eq!(nearby, Vec::<&Line>::new());
    }

    #[test]
    fn zero_slope_line() {
        let line = &Line::builder().point(0.0, 0.0).point(100.0, 0.0).build();

        let grid = Grid::new(vec![*line], DEFAULT_CELL_SIZE);

        let nearby = grid.lines_near(Vector2D(0.0, 0.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(50.0, 0.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(-10.0, 0.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(110.0, 22.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(150.0, 0.0), 1);
        assert_eq!(nearby, Vec::<&Line>::new());

        let nearby = grid.lines_near(Vector2D(-30.0, 0.0), 1);
        assert_eq!(nearby, Vec::<&Line>::new());
    }

    #[test]
    fn positive_slope_line() {
        let line = &Line::builder()
            .point(45.124, 98.348952734)
            .point(435.47457, 348.3489237)
            .build();

        let grid = Grid::new(vec![*line], DEFAULT_CELL_SIZE);

        let nearby = grid.lines_near(Vector2D(0.0, 0.0), 1);
        assert_eq!(nearby, Vec::<&Line>::new());

        let nearby = grid.lines_near(Vector2D(100.0, 125.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(400.0, 320.0), 1);
        assert_eq!(nearby, vec![line]);
    }

    #[test]
    fn negative_slope_line() {
        let line = &Line::builder()
            .point(45.124, 348.3489237)
            .point(435.47457, 98.348952734)
            .build();

        let grid = Grid::new(vec![*line], DEFAULT_CELL_SIZE);

        let nearby = grid.lines_near(Vector2D(0.0, 0.0), 1);
        assert_eq!(nearby, Vec::<&Line>::new());

        let nearby = grid.lines_near(Vector2D(200.0, 220.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(400.0, 100.0), 1);
        assert_eq!(nearby, vec![line]);
    }

    #[test]
    fn has_negative_x() {
        let line = &Line::builder()
            .point(-100.0, 0.0)
            .point(-10.0, 50.0)
            .build();

        let grid = Grid::new(vec![*line], DEFAULT_CELL_SIZE);

        let nearby = grid.lines_near(Vector2D(0.0, 0.0), 1);
        assert_eq!(nearby, Vec::<&Line>::new());

        let nearby = grid.lines_near(Vector2D(-100.0, 5.0), 1);
        assert_eq!(nearby, vec![line]);

        let nearby = grid.lines_near(Vector2D(-50.0, 25.0), 1);
        assert_eq!(nearby, vec![line]);
    }

    #[test]
    fn has_negative_y() {
        let line = &Line::builder()
            .point(0.0, -100.0)
            .point(150.0, 50.0)
            .build();

        let grid = Grid::new(vec![*line], DEFAULT_CELL_SIZE);

        let nearby = grid.lines_near(Vector2D(0.0, 0.0), 1);
        assert_eq!(nearby, Vec::<&Line>::new());

        let nearby = grid.lines_near(Vector2D(50.0, -40.0), 1);
        assert_eq!(nearby, vec![line]);
    }

    #[test]
    fn has_negative_y_and_negative_slope() {
        let line = &Line::builder()
            .point(0.0, -100.0)
            .point(50.0, -250.0)
            .build();

        let grid = Grid::new(vec![*line], DEFAULT_CELL_SIZE);

        let nearby = grid.lines_near(Vector2D(0.0, 0.0), 1);
        assert_eq!(nearby, Vec::<&Line>::new());

        let nearby = grid.lines_near(Vector2D(25.0, -175.0), 1);
        assert_eq!(nearby, vec![line]);
    }

    #[test]
    fn multiple_lines() {
        let line1 = Line::builder().point(0.0, 0.0).point(100.0, 0.0).build();
        let line2 = Line::builder().point(1.0, 0.0).point(100.0, 0.0).build();
        let line3 = Line::builder().point(2.0, 0.0).point(100.0, 0.0).build();
        let far_line = Line::builder()
            .point(0.0, 1000.0)
            .point(100.0, 1000.0)
            .build();
        let grid = Grid::new(vec![line1, line2, line3, far_line], DEFAULT_CELL_SIZE);

        let lines = grid.lines_near(Vector2D(50.0, 0.0), 1);
        assert_eq!(
            HashSet::from_iter(lines),
            HashSet::from([&line1, &line2, &line3])
        );
    }

    #[test]
    fn multiple_lines_with_remove() {
        let line1 = Line::builder().point(0.0, 0.0).point(100.0, 0.0).build();
        let line2 = Line::builder().point(1.0, 0.0).point(100.0, 0.0).build();
        let line3 = Line::builder().point(2.0, 0.0).point(100.0, 0.0).build();
        let far_line = Line::builder()
            .point(0.0, 1000.0)
            .point(100.0, 1000.0)
            .build();

        let mut grid = Grid::new(vec![line1, line2, line3, far_line], DEFAULT_CELL_SIZE);

        grid.remove_line(&line2);

        let lines = grid.lines_near(Vector2D(50.0, 0.0), 1);
        assert_eq!(HashSet::from_iter(lines), HashSet::from([&line1, &line3]));
    }

    #[test]
    fn multiple_lines_with_remove_last() {
        let line1 = Line::builder().point(0.0, 0.0).point(100.0, 0.0).build();
        let line2 = Line::builder().point(1.0, 0.0).point(100.0, 0.0).build();
        let line3 = Line::builder().point(2.0, 0.0).point(100.0, 0.0).build();
        let far_line = Line::builder()
            .point(0.0, 1000.0)
            .point(100.0, 1000.0)
            .build();

        let mut grid = Grid::new(vec![line1, line2, line3, far_line], DEFAULT_CELL_SIZE);

        grid.remove_line(&far_line);

        let lines = grid.lines_near(Vector2D(50.0, 0.0), 1);
        assert_eq!(
            HashSet::from_iter(lines),
            HashSet::from([&line1, &line2, &line3])
        );
    }

    #[test]
    fn all_lines_duplicates() {
        let line1 = Line::builder().point(0.0, 0.0).point(100.0, 0.0).build();
        let line2 = Line::builder().point(1.0, 0.0).point(100.0, 0.0).build();
        let line3 = Line::builder().point(2.0, 0.0).point(100.0, 0.0).build();
        let line4 = Line::builder().point(2.0, 0.0).point(100.0, 0.0).build();
        let far_line = Line::builder()
            .point(0.0, 1000.0)
            .point(100.0, 1000.0)
            .build();

        let grid = Grid::new(vec![line1, line2, line3, line4, far_line], DEFAULT_CELL_SIZE);

        let lines = grid.all_lines();
        assert_eq!(lines, &vec![line1, line2, line3, line4, far_line]);
    }

    #[test]
    fn lines_in_box() {
        let line = Line::builder().point(0.0, 0.0).point(100.0, 0.0).build();

        let grid = Grid::new(vec![line], DEFAULT_CELL_SIZE);

        let should_contain_line_cases = vec![
            grid.lines_near_box(Vector2D(-5.0, -5.0), Vector2D(5.0, 5.0)),
            grid.lines_near_box(Vector2D(0.0, 0.0), Vector2D(0.0, 0.0)),
            grid.lines_near_box(Vector2D(100.0, 0.0), Vector2D(100.0, 0.0)),
            grid.lines_near_box(Vector2D(50.0, 0.0), Vector2D(50.0, 0.0)),
            grid.lines_near_box(Vector2D(-30.0, -40.0), Vector2D(5.0, 20.0)),
            grid.lines_near_box(Vector2D(5.0, 20.0), Vector2D(-30.0, -40.0)),
        ];
        let should_not_contain_line_cases =
            vec![grid.lines_near_box(Vector2D(500.0, 0.0), Vector2D(550.0, 5.0))];

        for lines in should_contain_line_cases {
            assert_eq!(lines, vec![&line]);
        }

        for lines in should_not_contain_line_cases {
            assert_eq!(lines, Vec::<&Line>::new());
        }
    }

    #[test]
    fn correct_ordering() {
        let mut lines: Vec<Line> = vec![];
        let center = Vector2D(7.0, 7.0);
        for x in -1..=1 {
            for y in -1..=1 {
                let x = x as f64;
                let y = y as f64;
                let diff = Vector2D(x * DEFAULT_CELL_SIZE, y * DEFAULT_CELL_SIZE);
                lines.push(Line::builder().point_vec(center + diff).point_vec(center + diff + Vector2D(1.0, 1.0)).build());
            }
        }

        // perform some swaps just to make this a bit more unordered without making the test nondeterministic
        let mut lines_rand = lines.clone();
        lines_rand.swap(3, 8);
        lines_rand.swap(5, 0);
        lines_rand.swap(8, 5);
        lines_rand.swap(1, 6);
        lines_rand.push(Line::builder().point(100.0, 200.0).point(200.0, 200.0).build()); // also add a random line that shouldn't be in the output
        let grid = Grid::new(lines_rand, DEFAULT_CELL_SIZE);

        let near = grid.lines_near(Vector2D(0.5, 0.5), 1);
        assert_eq!(near, lines.iter().collect::<Vec<_>>());
    }
}
