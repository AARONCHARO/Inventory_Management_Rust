#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Item {
    id: String,
    name: String,
    description: String,
    quantity: u32,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Supplier {
    id: String,
    name: String,
    contact_info: String,
    items_supplied_ids: Vec<String>,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Order {
    id: String,
    item_id: String,
    quantity: u32,
    order_date: u64,
    supplier_id: String,
}

impl Storable for Item {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Item {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Supplier {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Supplier {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Order {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Order {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static ITEM_STORAGE: RefCell<StableBTreeMap<u64, Item, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static SUPPLIER_STORAGE: RefCell<StableBTreeMap<u64, Supplier, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static ORDER_STORAGE: RefCell<StableBTreeMap<u64, Order, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct ItemPayload {
    name: String,
    description: String,
    quantity: u32,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct SupplierPayload {
    name: String,
    contact_info: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct OrderPayload {
    item_id: String,
    quantity: u32,
    supplier_id: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Message {
    Success(String),
    Error(String),
    NotFound(String),
    InvalidPayload(String),
}

// Function to create a new inventory item
#[ic_cdk::update]
fn create_item(payload: ItemPayload) -> Result<Item, Message> {
    if payload.name.is_empty() || payload.description.is_empty() || payload.quantity == 0 {
        return Err(Message::InvalidPayload(
            "Ensure 'name', 'description', and 'quantity' are provided.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let item = Item {
        id: id.to_string(),
        name: payload.name,
        description: payload.description,
        quantity: payload.quantity,
        created_at: time(),
    };

    ITEM_STORAGE.with(|storage| storage.borrow_mut().insert(id, item.clone()));

    Ok(item)
}

// Function to create a new supplier
#[ic_cdk::update]
fn create_supplier(payload: SupplierPayload) -> Result<Supplier, Message> {
    if payload.name.is_empty() || payload.contact_info.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'name' and 'contact_info' are provided.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let supplier = Supplier {
        id: id.to_string(),
        name: payload.name,
        contact_info: payload.contact_info,
        items_supplied_ids: vec![],
        created_at: time(),
    };

    SUPPLIER_STORAGE.with(|storage| storage.borrow_mut().insert(id, supplier.clone()));

    Ok(supplier)
}

// Function to create a new order
#[ic_cdk::update]
fn create_order(payload: OrderPayload) -> Result<Order, Message> {
    if payload.item_id.is_empty() || payload.quantity == 0 || payload.supplier_id.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'item_id', 'quantity', and 'supplier_id' are provided.".to_string(),
        ));
    }

    // Validate if the item_id is valid
    let item = ITEM_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, item)| item.id == payload.item_id)
            .map(|(_, item)| item.clone())
    });
    if item.is_none() {
        return Err(Message::InvalidPayload(
            "Item ID is invalid.".to_string(),
        ));
    }

    // Validate if the supplier_id is valid
    let supplier = SUPPLIER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, supplier)| supplier.id == payload.supplier_id)
            .map(|(_, supplier)| supplier.clone())
    });
    if supplier.is_none() {
        return Err(Message::InvalidPayload(
            "Supplier ID is invalid.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let order = Order {
        id: id.to_string(),
        item_id: payload.item_id,
        quantity: payload.quantity,
        order_date: time(),
        supplier_id: payload.supplier_id,
    };

    ORDER_STORAGE.with(|storage| storage.borrow_mut().insert(id, order.clone()));

    Ok(order)
}

// Function to get all items
#[ic_cdk::query]
fn get_all_items() -> Vec<Item> {
    ITEM_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .map(|(_, item)| item.clone())
            .collect()
    })
}

// Function to get all suppliers
#[ic_cdk::query]
fn get_all_suppliers() -> Vec<Supplier> {
    SUPPLIER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .map(|(_, supplier)| supplier.clone())
            .collect()
    })
}

// Function to get all orders
#[ic_cdk::query]
fn get_all_orders() -> Vec<Order> {
    ORDER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .map(|(_, order)| order.clone())
            .collect()
    })
}

// Function to get an item by ID
#[ic_cdk::query]
fn get_item_by_id(item_id: String) -> Result<Item, Message> {
    ITEM_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, item)| item.id == item_id)
            .map(|(_, item)| item.clone())
            .ok_or(Message::NotFound("Item not found".to_string()))
    })
}

// Function to get a supplier by ID
#[ic_cdk::query]
fn get_supplier_by_id(supplier_id: String) -> Result<Supplier, Message> {
    SUPPLIER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, supplier)| supplier.id == supplier_id)
            .map(|(_, supplier)| supplier.clone())
            .ok_or(Message::NotFound("Supplier not found".to_string()))
    })
}

// Function to update an existing inventory item
#[ic_cdk::update]
fn update_item(item_id: String, payload: ItemPayload) -> Result<Item, Message> {
    ITEM_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let id = storage.iter().find(|(_, item)| item.id == item_id);
        match id {
            Some((key, _)) => {
                let updated_item = Item {
                    id: item_id.clone(),
                    name: payload.name,
                    description: payload.description,
                    quantity: payload.quantity,
                    created_at: time(),
                };
                storage.insert(key, updated_item.clone());
                Ok(updated_item)
            }
            None => Err(Message::NotFound("Item not found".to_string())),
        }
    })
}

// Function to update an existing supplier
#[ic_cdk::update]
fn update_supplier(supplier_id: String, payload: SupplierPayload) -> Result<Supplier, Message> {
    SUPPLIER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let id = storage.iter().find(|(_, supplier)| supplier.id == supplier_id);
        match id {
            Some((key, _)) => {
                let updated_supplier = Supplier {
                    id: supplier_id.clone(),
                    name: payload.name,
                    contact_info: payload.contact_info,
                    items_supplied_ids: vec![],
                    created_at: time(),
                };
                storage.insert(key, updated_supplier.clone());
                Ok(updated_supplier)
            }
            None => Err(Message::NotFound("Supplier not found".to_string())),
        }
    })
}

// Function to update an existing order
#[ic_cdk::update]
fn update_order(order_id: String, payload: OrderPayload) -> Result<Order, Message> {
    ORDER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let id = storage.iter().find(|(_, order)| order.id == order_id);
        match id {
            Some((key, _)) => {
                let updated_order = Order {
                    id: order_id.clone(),
                    item_id: payload.item_id,
                    quantity: payload.quantity,
                    order_date: time(),
                    supplier_id: payload.supplier_id,
                };
                storage.insert(key, updated_order.clone());
                Ok(updated_order)
            }
            None => Err(Message::NotFound("Order not found".to_string())),
        }
    })
}

// Function to delete an inventory item
#[ic_cdk::update]
fn delete_item(item_id: String) -> Result<(), Message> {
    ITEM_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let id = storage.iter().find(|(_, item)| item.id == item_id);
        match id {
            Some((key, _)) => {
                storage.remove(&key);
                Ok(())
            }
            None => Err(Message::NotFound("Item not found".to_string())),
        }
    })
}

// Function to delete a supplier
#[ic_cdk::update]
fn delete_supplier(supplier_id: String) -> Result<(), Message> {
    SUPPLIER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let id = storage.iter().find(|(_, supplier)| supplier.id == supplier_id);
        match id {
            Some((key, _)) => {
                storage.remove(&key);
                Ok(())
            }
            None => Err(Message::NotFound("Supplier not found".to_string())),
        }
    })
}

// Function to delete an order
#[ic_cdk::update]
fn delete_order(order_id: String) -> Result<(), Message> {
    ORDER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let id = storage.iter().find(|(_, order)| order.id == order_id);
        match id {
            Some((key, _)) => {
                storage.remove(&key);
                Ok(())
            }
            None => Err(Message::NotFound("Order not found".to_string())),
        }
    })
}

// Function to search for items by name
#[ic_cdk::query]
fn search_items_by_name(name: String) -> Vec<Item> {
    ITEM_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .filter(|(_, item)| item.name.contains(&name))
            .map(|(_, item)| item.clone())
            .collect()
    })
    
}


// Function to filter orders by supplier ID
#[ic_cdk::query]
fn filter_orders_by_supplier(supplier_id: String) -> Vec<Order> {
    ORDER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .filter(|(_, order)| order.supplier_id == supplier_id)
            .map(|(_, order)| order.clone())
            .collect()
    })
}

// Function to count the number of items
#[ic_cdk::query]
fn count_items() -> u64 {
    ITEM_STORAGE.with(|storage| storage.borrow().len() as u64)
}

// Function to count the number of suppliers
#[ic_cdk::query]
fn count_suppliers() -> u64 {
    SUPPLIER_STORAGE.with(|storage| storage.borrow().len() as u64)
}

// Function to count the number of orders
#[ic_cdk::query]
fn count_orders() -> u64 {
    ORDER_STORAGE.with(|storage| storage.borrow().len() as u64)
}

// Error types
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// need this to generate candid
ic_cdk::export_candid!();
