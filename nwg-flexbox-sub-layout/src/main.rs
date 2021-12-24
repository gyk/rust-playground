/*!
    A very simple application that show how to use nested flexbox layouts.
*/

use nwg::NativeUi;

#[derive(Default)]
pub struct FlexBoxApp {
    window: nwg::Window,

    main_layout: nwg::FlexboxLayout,

    toolbar_layout: nwg::FlexboxLayout,
    toolbar_left_layout: nwg::FlexboxLayout,
    button_l1: nwg::Button,
    button_l2: nwg::Button,
    toolbar_right_layout: nwg::FlexboxLayout,
    button_r1: nwg::Button,
    button_r2: nwg::Button,
    button_r3: nwg::Button,

    content_layout: nwg::FlexboxLayout,
    list_view: nwg::ListView,

    status_bar: nwg::StatusBar,
}

impl FlexBoxApp {
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

mod flexbox_app_ui {
    use super::*;
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    pub struct FlexBoxAppUi {
        inner: Rc<FlexBoxApp>,
        default_handler: RefCell<Option<nwg::EventHandler>>,
    }

    impl nwg::NativeUi<FlexBoxAppUi> for FlexBoxApp {
        fn build_ui(mut data: FlexBoxApp) -> Result<FlexBoxAppUi, nwg::NwgError> {
            use nwg::Event as E;

            // Controls
            nwg::Window::builder()
                .size((500, 500))
                .position((300, 300))
                .title("Flexbox sub-layout example")
                .build(&mut data.window)?;

            nwg::Button::builder()
                .text("L1")
                .parent(&data.window)
                .focus(true)
                .build(&mut data.button_l1)?;
            nwg::Button::builder()
                .text("L2")
                .parent(&data.window)
                .focus(true)
                .build(&mut data.button_l2)?;

            nwg::Button::builder()
                .text("R1")
                .parent(&data.window)
                .focus(true)
                .build(&mut data.button_r1)?;
            nwg::Button::builder()
                .text("R2")
                .parent(&data.window)
                .focus(true)
                .build(&mut data.button_r2)?;
            nwg::Button::builder()
                .text("R3")
                .parent(&data.window)
                .focus(true)
                .build(&mut data.button_r3)?;

            nwg::ListView::builder()
                .parent(&data.window)
                .item_count(10)
                .list_style(nwg::ListViewStyle::Detailed)
                .focus(true)
                .ex_flags(
                    nwg::ListViewExFlags::FULL_ROW_SELECT
                        | nwg::ListViewExFlags::AUTO_COLUMN_SIZE
                        | nwg::ListViewExFlags::BORDER_SELECT,
                )
                .build(&mut data.list_view)?;

            nwg::StatusBar::builder()
                .parent(&data.window)
                .text("Ready")
                .build(&mut data.status_bar)?;

            // Wrap-up
            let ui = FlexBoxAppUi {
                inner: Rc::new(data),
                default_handler: Default::default(),
            };

            // Events
            let evt_ui = Rc::downgrade(&ui.inner);
            let handle_events = move |evt, _evt_data, handle| {
                if let Some(evt_ui) = evt_ui.upgrade() {
                    match evt {
                        E::OnWindowClose => {
                            if &handle == &evt_ui.window {
                                FlexBoxApp::exit(&evt_ui);
                            }
                        }
                        _ => {}
                    }
                }
            };

            *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
                &ui.window.handle,
                handle_events,
            ));

            // Layout
            use nwg::stretch::{
                geometry::{Rect, Size},
                style::{AlignSelf, Dimension as D, FlexDirection, JustifyContent},
            };

            nwg::FlexboxLayout::builder()
                .parent(&ui.window)
                .flex_direction(FlexDirection::Row)
                .justify_content(JustifyContent::FlexStart)
                .child(&ui.button_l1)
                .child_size(Size {
                    width: D::Points(36.0),
                    height: D::Points(36.0),
                })
                .child(&ui.button_l2)
                .child_flex_grow(1.0)
                .child_size(Size {
                    width: D::Points(36.0),
                    height: D::Points(36.0),
                })
                .build_partial(&ui.toolbar_left_layout)?;

            nwg::FlexboxLayout::builder()
                .parent(&ui.window)
                .flex_direction(FlexDirection::Row)
                .justify_content(JustifyContent::FlexEnd)
                .child(&ui.button_r1)
                .child_size(Size {
                    width: D::Points(36.0),
                    height: D::Points(36.0),
                })
                .child(&ui.button_r2)
                .child_size(Size {
                    width: D::Points(36.0),
                    height: D::Points(36.0),
                })
                .child(&ui.button_r3)
                .child_size(Size {
                    width: D::Points(36.0),
                    height: D::Points(36.0),
                })
                .child_align_self(AlignSelf::Stretch)
                .build_partial(&ui.toolbar_right_layout)?;

            nwg::FlexboxLayout::builder()
                .parent(&ui.window)
                .flex_direction(FlexDirection::Row)
                .justify_content(JustifyContent::SpaceBetween)
                .padding(Rect {
                    start: D::Points(0.0),
                    end: D::Points(0.0),
                    top: D::Points(0.0),
                    bottom: D::Points(10.0),
                })
                // Left
                .child_layout(&ui.toolbar_left_layout)
                .child_size(Size {
                    width: D::Percent(50.0),
                    height: D::Points(40.0),
                })
                // Right
                .child_layout(&ui.toolbar_right_layout)
                .child_size(Size {
                    width: D::Percent(50.0),
                    height: D::Points(40.0),
                })
                .build_partial(&ui.toolbar_layout)?;

            nwg::FlexboxLayout::builder()
                .parent(&ui.window)
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::Center)
                .child(&ui.list_view)
                .child_margin(Rect {
                    start: D::Points(10.0),
                    end: D::Points(10.0),
                    top: D::Points(20.0),
                    bottom: D::Points(22.0),
                })
                .child_size(Size {
                    width: D::Auto,
                    height: D::Points(2000.0),
                })
                // .child_flex_shrink(2.0)
                .child_align_self(AlignSelf::Stretch)
                .build_partial(&ui.content_layout)?;

            nwg::FlexboxLayout::builder()
                .parent(&ui.window)
                .flex_direction(FlexDirection::Column)
                .child_layout(&ui.toolbar_layout)
                .child_margin(Rect {
                    start: D::Points(0.0),
                    end: D::Points(0.0),
                    top: D::Points(0.0),
                    bottom: D::Points(10.0),
                })
                .child_size(Size {
                    width: D::Auto,
                    height: D::Points(40.0),
                })
                .child_layout(&ui.content_layout)
                .child_margin(Rect {
                    start: D::Points(0.0),
                    end: D::Points(0.0),
                    top: D::Points(10.0),
                    bottom: D::Points(10.0),
                })
                .child_align_self(AlignSelf::Stretch)
                .child_size(Size {
                    width: D::Auto,
                    height: D::Auto,
                })
                .child(&ui.status_bar)
                .build(&ui.main_layout)?;

            Ok(ui)
        }
    }

    impl Drop for FlexBoxAppUi {
        /// To make sure that everything is freed without issues, the default handler must be unbound.
        fn drop(&mut self) {
            let handler = self.default_handler.borrow();
            if handler.is_some() {
                nwg::unbind_event_handler(handler.as_ref().unwrap());
            }
        }
    }

    impl Deref for FlexBoxAppUi {
        type Target = FlexBoxApp;

        fn deref(&self) -> &FlexBoxApp {
            &self.inner
        }
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let _ui = FlexBoxApp::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
