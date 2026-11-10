//! Clique no preview (`object-fit: contain`) → `input tap` no projetor.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}

pub const HY300_SCREEN: ScreenSize = ScreenSize {
    width: 1280,
    height: 720,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreviewBox {
    pub width: f64,
    pub height: f64,
}

/// Mapeia clique no elemento do preview para pixel da tela.
/// `None` se o clique caiu no letterbox.
pub fn map_tap(
    click_x: f64,
    click_y: f64,
    preview: PreviewBox,
    screen: ScreenSize,
) -> Option<(i32, i32)> {
    if preview.width <= 0.0 || preview.height <= 0.0 || screen.width == 0 || screen.height == 0 {
        return None;
    }
    let scale =
        (preview.width / f64::from(screen.width)).min(preview.height / f64::from(screen.height));
    if scale <= 0.0 {
        return None;
    }
    let content_w = f64::from(screen.width) * scale;
    let content_h = f64::from(screen.height) * scale;
    let off_x = (preview.width - content_w) / 2.0;
    let off_y = (preview.height - content_h) / 2.0;
    if click_x < off_x
        || click_y < off_y
        || click_x > off_x + content_w
        || click_y > off_y + content_h
    {
        return None;
    }
    let x = ((click_x - off_x) / scale).round() as i32;
    let y = ((click_y - off_y) / scale).round() as i32;
    let max_x = screen.width.saturating_sub(1) as i32;
    let max_y = screen.height.saturating_sub(1) as i32;
    Some((x.clamp(0, max_x), y.clamp(0, max_y)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    fn fake_screen() -> ScreenSize {
        ScreenSize {
            width: (320u32..3840u32).fake(),
            height: (180u32..2160u32).fake(),
        }
    }

    #[test]
    fn center_click_on_matching_box_hits_center_pixel() {
        let screen = HY300_SCREEN;
        let preview = PreviewBox {
            width: f64::from(screen.width),
            height: f64::from(screen.height),
        };
        let (x, y) = map_tap(preview.width / 2.0, preview.height / 2.0, preview, screen).unwrap();
        assert_eq!(x, (screen.width / 2) as i32);
        assert_eq!(y, (screen.height / 2) as i32);
    }

    #[test]
    fn origin_click_is_zero() {
        let screen = fake_screen();
        let preview = PreviewBox {
            width: f64::from(screen.width),
            height: f64::from(screen.height),
        };
        assert_eq!(map_tap(0.0, 0.0, preview, screen), Some((0, 0)));
    }

    #[test]
    fn letterbox_top_is_ignored() {
        let screen = HY300_SCREEN;
        let preview = PreviewBox {
            width: 1280.0,
            height: 900.0,
        };
        assert_eq!(map_tap(640.0, 10.0, preview, screen), None);
    }

    #[test]
    fn letterbox_side_is_ignored() {
        let screen = HY300_SCREEN;
        let preview = PreviewBox {
            width: 1600.0,
            height: 720.0,
        };
        assert_eq!(map_tap(10.0, 360.0, preview, screen), None);
    }

    #[test]
    fn far_corner_saturates() {
        let screen = HY300_SCREEN;
        let preview = PreviewBox {
            width: 1280.0,
            height: 720.0,
        };
        let (x, y) = map_tap(1280.0, 720.0, preview, screen).unwrap();
        assert_eq!(x, 1279);
        assert_eq!(y, 719);
    }

    #[test]
    fn empty_preview_is_none() {
        let screen = fake_screen();
        assert_eq!(
            map_tap(
                1.0,
                1.0,
                PreviewBox {
                    width: 0.0,
                    height: 100.0
                },
                screen
            ),
            None
        );
    }
}
