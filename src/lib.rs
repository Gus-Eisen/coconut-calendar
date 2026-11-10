use chk::{App, Assets, ChkTheme, Color, Context, Icons, RootInfo, Theme};
use prism::drawable::Drawable;
use ramp::prism;

mod calendar;
pub use calendar::*;
mod month;
pub use month::*;

ramp::run!(CoconutCalendar);

pub struct CoconutCalendar;

impl prism::Application for CoconutCalendar {
    fn name() -> String {
        "Coconut Calendar".into()
    }

    fn new(ctx: &mut Context) -> impl Drawable {
        CoconutCalendar.build(ctx, Assets::new(vec![]))
    }
}

impl App for CoconutCalendar {
    fn roots(&self, ctx: &mut Context, theme: &Theme) -> Vec<RootInfo> {
        let home = MonthScreen::root(ctx);
        vec![RootInfo::icon(ctx, theme, Icons::Home, "Home", home)]
    }

    fn theme(&self) -> ChkTheme {
        ChkTheme::Dark(Color::from_hex("#eb343a", 255))
    }
}
