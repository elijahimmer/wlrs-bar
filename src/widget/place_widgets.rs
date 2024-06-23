use super::*;

pub fn stack_widgets_right(
    widgets: &mut [impl std::ops::DerefMut<Target = dyn Widget>],
    area: Rect,
) {
    let Point {
        y: max_height,
        x: max_width,
    } = area.size();

    let des_widths = widgets
        .iter()
        .map(|w| w.desired_width(max_height))
        .collect::<Vec<u32>>();

    let total_width: u32 = des_widths.iter().sum();

    let des_widths = if total_width > max_width {
        let scale_factor = max_width as f32 / total_width as f32;
        let new_width = (total_width as f32 * scale_factor).round() as u32;
        assert!(new_width <= max_width);

        des_widths
            .into_iter()
            .map(|w| (w as f32 * scale_factor) as u32)
            .collect::<Vec<u32>>()
    } else {
        des_widths
    };

    let mut starting_from = area.min;

    let areas = des_widths.into_iter().map(|w| {
        let new_area = Rect::new(
            starting_from,
            Point {
                x: starting_from.x + w,
                y: area.max.y,
            },
        );
        assert!(area.contains_rect(new_area));
        starting_from = starting_from.x_shift(i32::try_from(w).unwrap());
        new_area
    });

    widgets
        .iter_mut()
        .zip(areas)
        .for_each(|(ref mut w, new_area)| {
            w.resize(new_area);
        })
}

/// stack widgets, on after another, from the right to the left.
pub fn stack_widgets_left(
    widgets: &mut [impl std::ops::DerefMut<Target = dyn Widget>],
    area: Rect,
) {
    let Point {
        y: max_height,
        x: max_width,
    } = area.size();

    let des_widths = widgets
        .iter()
        .map(|w| w.desired_width(max_height))
        .collect::<Vec<u32>>();

    let total_width: u32 = des_widths.iter().sum();

    let des_widths = if total_width > max_width {
        let scale_factor = max_width as f32 / total_width as f32;
        let new_width = (total_width as f32 * scale_factor).round() as u32;
        assert!(new_width <= max_width);

        des_widths
            .into_iter()
            .map(|w| (w as f32 * scale_factor) as u32)
            .collect::<Vec<u32>>()
    } else {
        des_widths
    };

    let mut starting_from = area.max;

    let areas = des_widths.into_iter().map(|w| {
        let new_area = Rect::new(
            starting_from,
            Point {
                x: starting_from.x - w,
                y: area.min.y,
            },
        );
        log::trace!("stack_widgets_left :: new_area: {new_area}, max_area: {area}");
        assert!(area.contains_rect(new_area));
        starting_from = starting_from.x_shift(-(i32::try_from(w).unwrap()));
        new_area
    });

    widgets.iter_mut().zip(areas).for_each(|(ref mut w, area)| {
        w.resize(area);
    })
}

/// places widgets from the center propagating out,
/// scaling all down by the same ratio if needed.
/// the widgets are places the center first, then left and right.
/// if there is a even amount, 2 are placed with edges on the center line.
pub fn center_widgets(widgets: &mut [impl std::ops::DerefMut<Target = dyn Widget>], area: Rect) {
    let (width_max, height_max) = (area.width(), area.height());
    log::trace!("center_widgets :: {area}");
    let mut widths: Vec<_> = widgets
        .iter()
        .map(|w| w.desired_width(height_max))
        .collect();
    let width_total: u32 = widths.iter().sum();

    if width_total > width_max {
        let ratio = width_max / width_total;

        widths.iter_mut().for_each(|w| (*w) *= ratio);
    }

    let mut iter = (0..)
        .map(|i| i % 2 == 0)
        .zip(widgets.iter_mut().zip(widths.iter()));

    let mut left = Rect::new(
        area.min,
        area.min
            + Point {
                x: width_max / 2,
                y: height_max,
            },
    );
    let mut right = Rect::new(
        area.min
            + Point {
                x: width_max / 2,
                y: 0,
            },
        area.max,
    );
    log::trace!("center_widgets :: left: {left}, right: {right}");

    if widths.len() % 2 == 1 {
        // is odd
        let (_, (widget, &width)) = iter.next().unwrap();
        let rect = area.place_at(
            Point {
                x: width,
                y: height_max,
            },
            Align::Center,
            Align::Center,
        );
        log::trace!("center_widgets :: rect: {rect}, width: {width}");

        widget.resize(rect);

        left.max.x -= rect.width() / 2;
        right.min.x += rect.width() / 2;
        assert!(left.min.x <= left.max.x);
        assert!(right.min.x <= right.max.x);
    };
    log::trace!("center_widgets :: left: {left}, right: {right}");

    iter.for_each(|(go_left, (widget, &width))| {
        let rect = if go_left {
            left.place_at(
                Point {
                    x: width,
                    y: height_max,
                },
                Align::End,
                Align::Center,
            )
        } else {
            right.place_at(
                Point {
                    x: width,
                    y: height_max,
                },
                Align::Start,
                Align::Center,
            )
        };

        widget.resize(rect);

        if go_left {
            left.max.x -= rect.width();
        } else {
            right.min.x += rect.width();
        }
    });
}
