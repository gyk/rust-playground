use std::cell::RefCell;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

use nwd::NwgUi;
use nwg::{NativeUi, NoticeSender};

// This example demonstrates how bad it is to run a child window in a separate thread.

#[derive(Default, NwgUi)]
pub struct DataViewPopover {
    #[nwg_control(size: (500, 350), title: "DataView - Animals list", flags: "WINDOW")]
    #[nwg_events(OnWindowClose: [DataViewPopover::exit], OnInit: [DataViewPopover::load_data])]
    window: nwg::Window,

    #[nwg_resource(family: "Arial", size: 19)]
    arial: nwg::Font,

    #[nwg_resource(initial: 5)]
    view_icons: nwg::ImageList,

    #[nwg_resource(initial: 5, size: (16, 16))]
    view_icons_small: nwg::ImageList,

    #[nwg_layout(parent: window)]
    layout: nwg::GridLayout,

    #[nwg_control(item_count: 10, size: (500, 350), list_style: nwg::ListViewStyle::Detailed, focus: true,
        ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT,
    )]
    #[nwg_layout_item(layout: layout, col: 0, col_span: 4, row: 0, row_span: 6)]
    data_view: nwg::ListView,

    #[nwg_control(text: "View:", font: Some(&data.arial))]
    #[nwg_layout_item(layout: layout, col: 4, row: 0)]
    label: nwg::Label,

    #[nwg_control(collection: vec!["Simple", "Details", "Icon", "Icon small"], selected_index: Some(1), font: Some(&data.arial))]
    #[nwg_layout_item(layout: layout, col: 4, row: 1)]
    #[nwg_events(OnComboxBoxSelection: [DataViewPopover::update_view])]
    view_style: nwg::ComboBox<&'static str>,

    #[nwg_control]
    #[nwg_events(OnNotice: [DataViewPopover::check_signal])]
    data_view_notice: nwg::Notice,

    #[nwg_control(text: "Open")]
    #[nwg_layout_item(layout: layout, col: 4, row: 2)]
    #[nwg_events(OnButtonClick: [DataViewPopover::open_file])]
    button: nwg::Button,

    notice_sender: Option<nwg::NoticeSender>,

    data_sender: Option<mpsc::Sender<String>>,
    signal_receiver: Option<mpsc::Receiver<()>>,
}

impl DataViewPopover {
    fn load_data(&self) {
        let dv = &self.data_view;
        let icons = &self.view_icons;
        let icons_small = &self.view_icons_small;

        // Load the listview images
        icons.add_icon_from_filename("./test_rc/cog.ico").unwrap();
        icons.add_icon_from_filename("./test_rc/love.ico").unwrap();
        icons_small
            .add_icon_from_filename("./test_rc/cog.ico")
            .unwrap();
        icons_small
            .add_icon_from_filename("./test_rc/love.ico")
            .unwrap();

        // Setting up the listview data
        dv.set_image_list(Some(icons), nwg::ListViewImageListType::Normal);
        dv.set_image_list(Some(icons_small), nwg::ListViewImageListType::Small);

        dv.insert_column("Name");
        dv.insert_column(nwg::InsertListViewColumn {
            index: Some(1),
            fmt: Some(nwg::ListViewColumnFlags::RIGHT),
            width: Some(60),
            text: Some("test".into()),
        });
        dv.set_headers_enabled(true);

        // Passing a str to this method will automatically push the item at the end of the list in the first column
        dv.insert_item("Cat");
        dv.insert_item(nwg::InsertListViewItem {
            index: Some(0),
            column_index: 1,
            text: Some("Felis".into()),
            image: None,
        });

        // To insert a new row, use the index 0.
        dv.insert_item(nwg::InsertListViewItem {
            index: Some(0),
            column_index: 0,
            text: Some("Moose".into()),
            image: Some(1),
        });

        dv.insert_item(nwg::InsertListViewItem {
            index: Some(0),
            column_index: 1,
            text: Some("Alces".into()),
            image: None,
        });

        // Insert multiple item on a single row.
        dv.insert_items_row(None, &["Dog", "Canis"]);

        // Insert many item at one
        dv.insert_items(&["Duck", "Horse", "Boomalope"]);
        dv.insert_items(&[
            nwg::InsertListViewItem {
                index: Some(3),
                column_index: 1,
                text: Some("Anas".into()),
                image: None,
            },
            nwg::InsertListViewItem {
                index: Some(4),
                column_index: 1,
                text: Some("Equus".into()),
                image: None,
            },
        ]);

        // Update items
        dv.update_item(
            2,
            nwg::InsertListViewItem {
                image: Some(1),
                ..Default::default()
            },
        );
        dv.update_item(
            4,
            nwg::InsertListViewItem {
                image: Some(1),
                ..Default::default()
            },
        );
    }

    fn update_view(&self) {
        let value = self.view_style.selection_string();

        let style = match value.as_ref().map(|v| v as &str) {
            Some("Icon") => nwg::ListViewStyle::Icon,
            Some("Icon small") => nwg::ListViewStyle::SmallIcon,
            Some("Details") => nwg::ListViewStyle::Detailed,
            None | Some(_) => nwg::ListViewStyle::Simple,
        };

        self.data_view.set_list_style(style);
    }

    fn popup(
        notice_sender: nwg::NoticeSender,
        data_sender: mpsc::Sender<String>,
        signal_receiver: mpsc::Receiver<()>,
        back_notice_sender: Arc<Mutex<Option<nwg::NoticeSender>>>,
        x: i32,
        y: i32,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            // Create the UI just like in the main function
            let popover = Self {
                notice_sender: Some(notice_sender),
                data_sender: Some(data_sender),
                signal_receiver: Some(signal_receiver),
                ..Default::default()
            };
            let popover = DataViewPopover::build_ui(popover).expect("Failed to build data view");
            *back_notice_sender.lock().unwrap() = Some(popover.data_view_notice.sender());
            let (w, h) = popover.window.size();
            popover.window.set_position(x - w as i32, y - h as i32);
            popover.window.set_visible(true);
            nwg::dispatch_thread_events();
        })
    }

    fn open_file(&self) {
        // Send user's selection
        if let Some(data_sender) = &self.data_sender {
            if let Some(r) = self.data_view.selected_item() {
                let c = self.data_view.selected_column();
                if let Some(item) = self.data_view.item(r, c, 256) {
                    let _ = data_sender.send(item.text);
                }
            }
        };

        if let Some(sender) = self.notice_sender {
            sender.notice();
        }
    }

    fn check_signal(&self) {
        if let Some(signal_rx) = &self.signal_receiver {
            if let Ok(()) = signal_rx.recv() {
                self.exit();
            }
        }
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

#[derive(Default, NwgUi)]
pub struct SystemTray {
    #[nwg_control]
    window: nwg::MessageWindow,

    #[nwg_resource(source_file: Some("./test_rc/cog.ico"))]
    icon: nwg::Icon,

    #[nwg_control(icon: Some(&data.icon), tip: Some("Hello"))]
    #[nwg_events(MousePressLeftUp: [SystemTray::show_popover], OnContextMenu: [SystemTray::show_menu])]
    tray: nwg::TrayNotification,

    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,

    #[nwg_control(parent: tray_menu, text: "Hello")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::hello1])]
    tray_item1: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Popup")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::hello2])]
    tray_item2: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Pop-over list")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::show_popover])]
    tray_item3: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::exit])]
    tray_item4: nwg::MenuItem,

    #[nwg_control]
    #[nwg_events(OnNotice: [SystemTray::set_data_view_string, SystemTray::hello2])]
    tray_notice: nwg::Notice,

    data_view_handle: RefCell<Option<JoinHandle<()>>>,
    data_view_string: RefCell<Option<String>>,
    data_view_receiver: RefCell<Option<mpsc::Receiver<String>>>,
    signal_sender: RefCell<Option<mpsc::Sender<()>>>,
    notice_sender: Arc<Mutex<Option<NoticeSender>>>,
}

impl SystemTray {
    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x - 10, y - 20);
    }

    fn show_popover(&self) {
        if self.data_view_handle.borrow().is_some() {
            if let Some(signal_sender) = &*self.signal_sender.borrow() {
                let _ = signal_sender.send(());
            }
        }

        let (x, y) = nwg::GlobalCursor::position();
        let (tx, rx) = mpsc::channel();
        let (signal_tx, signal_rx) = mpsc::channel();
        let j = DataViewPopover::popup(
            self.tray_notice.sender(),
            tx,
            signal_rx,
            Arc::clone(&self.notice_sender),
            x - 10,
            y - 20,
        );
        self.data_view_handle.borrow_mut().replace(j);
        *self.data_view_receiver.borrow_mut() = Some(rx);
        *self.signal_sender.borrow_mut() = Some(signal_tx);
    }

    fn set_data_view_string(&self) {
        if let Some(data_view_receiver) = &*self.data_view_receiver.borrow() {
            if let Ok(s) = data_view_receiver.recv() {
                self.data_view_string.borrow_mut().replace(s);
            }
        }
    }

    fn hello1(&self) {
        nwg::simple_message("Hello", "Hello World!");
    }

    fn hello2(&self) {
        let flags = nwg::TrayNotificationFlags::USER_ICON | nwg::TrayNotificationFlags::LARGE_ICON;
        let message = format!(
            "Hello World, {}",
            self.data_view_string.borrow().as_deref().unwrap_or("")
        );
        self.tray.show(
            &message,
            Some("Welcome to my application"),
            Some(flags),
            Some(&self.icon),
        );
    }

    fn exit(&self) {
        if let Some(signal_sender) = &*self.signal_sender.borrow() {
            let _ = signal_sender.send(());
        }

        if let Some(notice_sender) = *self.notice_sender.lock().unwrap() {
            notice_sender.notice();
        }

        if let Some(handle) = self.data_view_handle.borrow_mut().take() {
            let _ = handle.join();
        }
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let _ui = SystemTray::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
