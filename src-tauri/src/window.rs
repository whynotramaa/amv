pub fn clamp_position(
    position: (i32, i32),
    size: (u32, u32),
    area: (i32, i32, u32, u32),
) -> (i32, i32) {
    let clamp = |value: i32, origin: i32, available: u32, extent: u32| {
        let maximum = i64::from(origin) + i64::from(available.saturating_sub(extent));
        i64::from(value).clamp(i64::from(origin), maximum) as i32
    };
    (
        clamp(position.0, area.0, area.2, size.0),
        clamp(position.1, area.1, area.3, size.1),
    )
}

#[cfg(test)]
mod tests {
    use super::clamp_position;

    #[test]
    fn keeps_window_in_work_area_including_negative_monitor_positions() {
        assert_eq!(
            clamp_position((2000, 1000), (520, 600), (0, 0, 1920, 1040)),
            (1400, 440)
        );
        assert_eq!(
            clamp_position((-1800, -100), (520, 600), (-1920, 0, 1920, 1040)),
            (-1800, 0)
        );
        assert_eq!(
            clamp_position((40, 40), (520, 600), (0, 0, 400, 300)),
            (0, 0)
        );
    }
}
