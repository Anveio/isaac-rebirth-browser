//! MenuManager presentation: the view that pans between menu pages, the
//! Back/Select widgets, File select's paper animations and the Character
//! carousel. Every page is drawn at its own origin plus the view, so a page
//! change scrolls the whole board instead of cutting.
use serde::{Deserialize, Serialize};

use crate::Screen;

/// MenuManager+38 page ids that the port reaches.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Page {
    Title,
    Save,
    Game,
    Character,
    Challenge,
}

impl Page {
    pub fn of(screen: Screen) -> Option<Self> {
        match screen {
            Screen::Title => Some(Self::Title),
            Screen::Profile => Some(Self::Save),
            Screen::Main => Some(Self::Game),
            Screen::Character | Screen::Seed => Some(Self::Character),
            Screen::Challenge => Some(Self::Challenge),
            Screen::Playing => None,
        }
    }
}

/// SaveSelectMenu.anm2 clips and the per-file drawing clip.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Clip {
    pub animation: String,
    pub frame: u32,
    pub playing: bool,
}

impl Clip {
    fn play(animation: &str) -> Self {
        Self {
            animation: animation.into(),
            frame: 0,
            playing: true,
        }
    }
    /// Sprite update40F9F0 on an authored clip of `frames` frames.
    fn update(&mut self, frames: u32, looping: bool) {
        if !self.playing {
            return;
        }
        if self.frame + 1 < frames {
            self.frame += 1;
        } else if looping {
            self.frame = 0;
        } else {
            self.playing = false;
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveSelect {
    /// Selected file, 1-based (save menu +4 is its zero-based index).
    pub file: u8,
    pub menu: Clip,
    pub drawings: [Clip; 3],
}

impl SaveSelect {
    /// 5A8B80: every sprite Idle, then the selected file's Select and drawing.
    fn reset(file: u8) -> Self {
        let mut drawings = [Clip::play("Idle"), Clip::play("Idle"), Clip::play("Idle")];
        drawings[usize::from(file - 1)] = Clip::play("Animated");
        Self {
            file,
            menu: Clip::play(&format!("File{file}Select")),
            drawings,
        }
    }
    /// The selection input plays the old file's Deselect and stills its drawing.
    pub fn select(&mut self, file: u8) {
        if file == self.file {
            return;
        }
        self.menu = Clip::play(&format!("File{}Deselect", self.file));
        self.drawings[usize::from(self.file - 1)] = Clip::play("Idle");
        self.file = file;
    }
    /// 5A9490 on an even System counter: sprite updates, then a finished
    /// non-Select clip hands over to the selected file's Select and drawing.
    fn update(&mut self) {
        // SelectN/DeselectN author five frames; Idle one; Animated loops ten.
        self.menu
            .update(if self.menu.animation == "Idle" { 1 } else { 5 }, false);
        for drawing in &mut self.drawings {
            let animated = drawing.animation == "Animated";
            drawing.update(if animated { 10 } else { 1 }, animated);
        }
        let select = format!("File{}Select", self.file);
        if !self.menu.playing && self.menu.animation != select {
            self.menu = Clip::play(&select);
            self.drawings[usize::from(self.file - 1)] = Clip::play("Animated");
        }
    }
}

/// Character menu +1458 rotation and +1460 portrait positions.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Carousel {
    pub angle: f32,
    pub portraits: Vec<[f32; 2]>,
}

impl Carousel {
    /// 5A0680 fills the portraits with the screen centre and zeroes the angle.
    fn reset(count: usize) -> Self {
        Self {
            angle: 0.0,
            portraits: vec![[240.0, 135.0]; count],
        }
    }
    /// 59DD20: every portrait first turns the shared angle a bounded step
    /// toward the selected slot, then takes its position at that angle.
    fn update(&mut self, selection: usize) {
        let count = self.portraits.len();
        if count == 0 {
            return;
        }
        let total = count as f32;
        let step = 360.0_f32 / total;
        let slot = (count - selection % count) % count;
        let target = slot as f32 * step;
        let limit = step / 75.0;
        for index in 0..count {
            let angle = self.angle;
            if target != angle {
                let (difference, forward) = if target > angle {
                    (target - angle, true)
                } else {
                    (angle - target, false)
                };
                // The shorter way round turns toward the target.
                let toward = difference < 180.0;
                let turn = difference.min(limit);
                self.angle = if toward == forward {
                    angle + turn
                } else {
                    angle - turn
                };
                if self.angle < 0.0 {
                    self.angle = 360.0;
                } else if self.angle > 360.0 {
                    self.angle = 0.0;
                }
            }
            // The f64 portrait angle narrows to f32 before sincosf.
            let around =
                -((f64::from(index as f32) * std::f64::consts::TAU / f64::from(total)) as f32);
            let (x, y) = (
                crate::fmath::cos(around) * 70.0,
                crate::fmath::sin(around) * 70.0,
            );
            // Vector rotate612640 converts degrees with f32 0x3c8efa35.
            let radians = (90.0 - self.angle) * f32::from_bits(0x3c8e_fa35);
            let (sin, cos) = (crate::fmath::sin(radians), crate::fmath::cos(radians));
            let (x, y) = (x * cos - y * sin + 240.0, sin * x + cos * y + 70.0);
            let x = x - (240.0 - x) / 5.0;
            let y = (135.0 - y) / 1.5 + y;
            self.portraits[index] = [x, y];
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MenuPresentation {
    /// MenuManager+3C: pages draw at their origin plus this view.
    pub view: [f32; 2],
    /// MenuManager+0: SetMenu between two real pages pans instead of cutting.
    pub moving: bool,
    /// MenuManager+38 as the view update sees it: the page selected before
    /// this System iteration's input.
    pub page: Page,
    /// 59CCD0 moves the Character page's target one screen right for seed entry.
    pub seed_view: bool,
    /// 59CBC0's Challenge target from the selected challenge.
    pub challenge_scroll: u32,
    /// MenuManager+21: widgets are shown until a key is pressed on a settled page.
    pub widgets_shown: bool,
    /// MenuManager+2C/+30 Back and Select widget drop below their rest.
    pub widgets: [f32; 2],
    pub save: SaveSelect,
    pub carousel: Carousel,
    /// Character menu+24: the hard-mode BloodStain alpha, 0.05 per render.
    pub blood: f32,
    /// MenuManager+28: the second MenuOverlay pass in hard mode, 0.025 per
    /// render up to one half.
    pub hard_overlay: f32,
    /// The difficulty widget frame, set by the even-tick sprite pass before
    /// that iteration's input.
    pub hard_widget: bool,
    hard_before: bool,
}

impl MenuPresentation {
    /// MenuInit59A4D0 followed by SetMenu from page 0, which never pans.
    pub fn new(page: Page, file: u8, characters: usize) -> Self {
        Self {
            view: [0.0, 0.0],
            moving: false,
            page,
            seed_view: false,
            challenge_scroll: 0,
            widgets_shown: true,
            widgets: [90.0, 90.0],
            save: SaveSelect::reset(file),
            carousel: Carousel::reset(characters),
            blood: 0.0,
            hard_overlay: 0.0,
            hard_widget: false,
            hard_before: false,
        }
    }

    /// The +88CF0 page positions (negated origins); 59CBC0 and 59CCD0 adjust
    /// the Challenge and Character entries.
    fn position(&self, page: Page) -> [f32; 2] {
        match page {
            Page::Title => [0.0, 0.0],
            Page::Save => [0.0, -270.0],
            Page::Game => [0.0, -810.0],
            Page::Character => [if self.seed_view { -960.0 } else { -480.0 }, -810.0],
            Page::Challenge => [480.0, challenge_y(self.challenge_scroll)],
        }
    }

    /// MenuManager update59B180's view step before the page update.
    fn step_view(&mut self) {
        let target = self.position(self.page);
        if !self.moving {
            self.view = target;
            return;
        }
        let delta = [target[0] - self.view[0], target[1] - self.view[1]];
        let length = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
        if length <= 1.0 {
            self.view = target;
            self.moving = false;
            return;
        }
        let inverse = 1.0 / length;
        let speed = (length * 0.1).max(1.0);
        self.view[0] += delta[0] * inverse * speed;
        self.view[1] += inverse * delta[1] * speed;
    }

    /// One MenuManager update and render for the screen after this
    /// iteration's input. `even` is the page sprites' System parity.
    pub fn advance(&mut self, screen: Screen, input: Input) {
        let Some(page) = Page::of(screen) else {
            return;
        };
        self.step_view();
        // 59B3F1: a key on a settled page (or any later page) hides the widgets.
        if self.widgets_shown
            && self.page != Page::Title
            && (!self.moving || !matches!(self.page, Page::Save))
            && input.key_held
        {
            self.widgets_shown = false;
        }
        match self.page {
            Page::Save if input.even => self.save.update(),
            Page::Character => self.carousel.update(input.character),
            _ => {}
        }
        self.set_page(page, input);
        self.step_widgets();
        self.step_hard(input.hard);
        if input.even {
            self.hard_widget = self.hard_before;
        }
        self.hard_before = input.hard;
    }

    /// 59C5B9 and 59FA6B: both hard-mode fades step on every render toward
    /// the selected difficulty, whichever page is shown.
    fn step_hard(&mut self, hard: bool) {
        if self.hard_overlay < 0.5 && hard {
            self.hard_overlay += 0.025;
        } else if self.hard_overlay > 0.0 && !hard {
            self.hard_overlay -= 0.025;
        }
        if self.blood < 1.0 && hard {
            self.blood += 0.05;
        } else if self.blood > 0.0 && !hard {
            self.blood -= 0.05;
        }
    }

    /// SetMenu59CAC0 and the page targets the page updates set.
    fn set_page(&mut self, page: Page, input: Input) {
        let seed_view = input.seed;
        let challenge_scroll = if page == Page::Challenge {
            input.challenge
        } else {
            0
        };
        if page != self.page {
            self.moving = true;
            // 5A0680 resets the Character carousel when Save opens Game.
            // Game back to Save (5A8B50) only reloads the selected file's
            // drawing sheet, so File select resumes its clips where it left.
            if (self.page, page) == (Page::Save, Page::Game) {
                self.carousel = Carousel::reset(self.carousel.portraits.len());
            }
            self.page = page;
        }
        if seed_view != self.seed_view || challenge_scroll != self.challenge_scroll {
            self.seed_view = seed_view;
            self.challenge_scroll = challenge_scroll;
            self.moving = true;
        }
    }

    /// 59B5E0 render: hidden widgets drop 6 per frame to 90; Title drops
    /// them and every other reached page raises them to 0.
    fn step_widgets(&mut self) {
        let (limit, step) = match self.page {
            Page::Title => (90.0, 6.0),
            _ if !self.widgets_shown => (90.0, 6.0),
            _ => (0.0, -6.0),
        };
        for offset in &mut self.widgets {
            if (step > 0.0 && *offset < limit) || (step < 0.0 && *offset > limit) {
                *offset += step;
            }
        }
    }

    /// MenuInit59A4D0 and SetMenu(Game) when a run returns to the menus.
    pub fn reinitialize(&mut self) {
        let hard = self.hard_before;
        *self = Self::new(Page::Game, self.save.file, self.carousel.portraits.len());
        (self.hard_widget, self.hard_before) = (hard, hard);
    }
}

/// 59CBC0: the first four challenges keep the page at its origin; later
/// ones scroll 18 per row, and past the fifteenth it rests at -1025.
fn challenge_y(selection: u32) -> f32 {
    if selection <= 4 {
        -810.0
    } else if selection <= 15 {
        -810.0 - (selection * 18 - 72) as f32
    } else {
        -1025.0
    }
}

/// The public facts one MenuManager iteration reads.
#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub even: bool,
    pub key_held: bool,
    pub character: usize,
    pub seed: bool,
    pub challenge: u32,
    pub hard: bool,
}
