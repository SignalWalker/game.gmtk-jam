use std::cell::RefCell;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use godot::{
    classes::{
        Control, IControl, Input, InputEvent, Label, class_macros::private::virtuals::Xrvrs::Gd,
    },
    obj::Singleton,
    prelude::{Base, GodotClass, OnReady, godot_api},
};

type DeviceId = i64;

struct PlayerSlot {
    name: String,
    devices: RefCell<HashSet<DeviceId>>,
}

impl PlayerSlot {
    fn new(name: String) -> Self {
        Self {
            name,
            devices: RefCell::new(HashSet::new()),
        }
    }

    fn remove_device(&self, id: DeviceId) {
        self.devices.borrow_mut().remove(&id);
    }
}

struct VersusSetupController {
    connected_devices: RefCell<HashMap<DeviceId, Option<usize>>>,
    slots: HashMap<usize, PlayerSlot>,
}

impl VersusSetupController {
    fn new(slots: usize) -> Self {
        let mut connected_devices = HashMap::new();
        connected_devices.extend(
            Input::singleton()
                .get_connected_joypads()
                .iter_shared()
                .map(|id| (id, None)),
        );
        Self {
            connected_devices: RefCell::new(connected_devices),
            slots: (0..slots)
                .into_iter()
                .map(|index| (index, PlayerSlot::new(format!("P{index}"))))
                .collect(),
        }
    }

    fn init_gui(&self, gui: &mut VersusSetupMenu) {
        gui.update_connected_devices(&self.connected_devices.borrow());
        gui.update_slots(&self.slots);
    }

    fn add_device(&self, gui: &mut VersusSetupMenu, id: DeviceId) {
        self.connected_devices.borrow_mut().insert(id, None);
        gui.update_connected_devices(&self.connected_devices.borrow());
    }

    fn remove_device(&self, gui: &mut VersusSetupMenu, id: DeviceId) {
        if let Some(slot_index) = self.connected_devices.borrow_mut().remove(&id).flatten() {
            self.slots[&slot_index].remove_device(id);
        };
        gui.update_connected_devices(&self.connected_devices.borrow());
    }

    fn set_device_slot(&self, gui: &mut VersusSetupMenu, device: DeviceId, slot: Option<usize>) {
        if let Some(old_slot) = self
            .connected_devices
            .borrow()
            .get(&device)
            .copied()
            .flatten()
        {
            self.slots[&old_slot].remove_device(device);
        }
        self.connected_devices.borrow_mut().insert(device, slot);
        gui.update_connected_devices(&self.connected_devices.borrow());
    }
}

#[derive(GodotClass)]
#[class(init, base = Control)]
pub struct VersusSetupMenu {
    base: Base<Control>,

    #[init(val = Rc::new(VersusSetupController::new(2)))]
    controller: Rc<VersusSetupController>,

    #[init(node = "%SlotContainer")]
    slot_container: OnReady<Gd<Control>>,

    slot_controls: HashMap<usize, Gd<Control>>,

    device_icons: HashMap<DeviceId, Gd<Label>>,
}

impl VersusSetupMenu {
    fn add_slot(&mut self, index: usize, slot: &PlayerSlot) {
        let slot_con = VersusSlot::new(index, slot);
        self.slot_container.add_child(&slot_con);
    }

    fn update_connected_devices(&mut self, devices: &HashMap<DeviceId, Option<usize>>) {
        for (device, slot) in devices {}
    }

    fn update_slots(&mut self, slots: &HashMap<usize, PlayerSlot>) {}
}

#[godot_api]
impl IControl for VersusSetupMenu {
    fn enter_tree(&mut self) {
        let con = self.controller.clone();
        Input::singleton()
            .signals()
            .joy_connection_changed()
            .builder()
            .connect_other_gd(
                self,
                move |mut menu: Gd<VersusSetupMenu>, id: DeviceId, connected: bool| {
                    if connected {
                        con.add_device(&mut menu.bind_mut(), id);
                    } else {
                        con.remove_device(&mut menu.bind_mut(), id);
                    }
                },
            );
    }

    fn ready(&mut self) {
        self.controller.clone().init_gui(self);
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        let evdev = event.get_device();
        todo!();
    }
}

#[derive(GodotClass)]
#[class(no_init, base = Control)]
pub struct VersusSlot {
    base: Base<Control>,

    index: usize,
}

impl VersusSlot {
    fn new(index: usize, slot: &PlayerSlot) -> Gd<Self> {
        Gd::from_init_fn(|base| Self { base, index })
    }
}
