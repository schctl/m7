//! Pane collection widget.

use iced::widget::pane_grid::{self, Pane as PaneId};
use iced::widget::{button, text};

use super::{tab, Pane, Tab};
use crate::gui::Theme;

/// Grid split option.
#[derive(Debug, Clone)]
pub struct Split {
    /// Axis along which the new split must occur.
    axis: pane_grid::Axis,
    /// Id of the pane to split.
    ///
    /// If [`None`], is the focused pane.
    pane: Option<PaneId>,
}

/// Messages from the controller section of each pane.
#[derive(Debug, Clone)]
pub enum ControlMessage {
    Split(Split),
    Maximize(PaneId),
    Restore,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// A message to a single pane.
    Pane(super::Message, PaneId),
    Control(ControlMessage),
    // Grid messages
    // These and their handlers are more or less copied exactly from iced examples
    // https://github.com/iced-rs/iced/blob/master/examples/pane_grid/src/main.rs
    Clicked(PaneId),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
}

/// The main space where all panes are displayed.
pub struct Area {
    /// Panegrid internal state.
    panes: pane_grid::State<Pane>,
    /// Focused pane.
    focused: Option<PaneId>,
}

impl Area {
    pub fn new(pane: Pane) -> Self {
        let (panes, id) = pane_grid::State::new(pane);

        Self {
            panes,
            focused: Some(id),
        }
    }

    pub fn update(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::Pane(m, id) => {
                // Control messages are intended for the pane area
                // So we don't pass them onto the pane
                if let super::Message::Control(c) = m {
                    self.update(Message::Control(c))?;
                } else {
                    let pane = self.panes.get_mut(&id).unwrap();
                    pane.update(m)?;
                }
            }
            Message::Clicked(pane) => {
                self.focused = Some(pane);
            }
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(&split, ratio);
            }
            Message::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                self.panes.swap(&pane, &target);
            }
            Message::Control(p) => match p {
                ControlMessage::Maximize(pane) => self.panes.maximize(&pane),
                ControlMessage::Restore => self.panes.restore(),
                ControlMessage::Split(split) => {
                    if let Some(pane) = split.pane.map_or(self.focused, Some) {
                        let result = self.panes.split(split.axis, &pane, Pane::new(Tab::new()?));

                        if let Some((pane, _)) = result {
                            self.focused = Some(pane);
                        }
                    }
                }
            },
            _ => {}
        }

        Ok(())
    }

    pub fn view(&self) -> anyhow::Result<iced::Element<'_, Message, iced::Renderer<Theme>>> {
        let grid = pane_grid::PaneGrid::new(&self.panes, |id, state, _focused| {
            pane_grid::Content::new(
                state
                    .view(super::ViewOpts {
                        tab: tab::ViewOpts { columns: 6 },
                        controls: vec![button(text("split"))
                            .on_press(ControlMessage::Split(Split {
                                axis: pane_grid::Axis::Horizontal,
                                pane: Some(id),
                            }))
                            .into()],
                    })
                    .unwrap()
                    .map(move |m| Message::Pane(m, id)),
            )
        })
        .on_click(Message::Clicked)
        .on_drag(Message::Dragged)
        .on_resize(10, Message::Resized);

        Ok(grid.into())
    }
}
