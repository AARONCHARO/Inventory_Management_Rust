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
            counter.borrow_mut().set(current_value + 1).map(|_| current_value + 1)
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
            counter.borrow_mut().set(current_value + 1).map(|_| current_value + 1)
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
    let item_exists = ITEM_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, item)| item.id == payload.item_id)
    });
    if !item_exists {
        return Err(Message::InvalidPayload("Item ID is invalid.".to_string()));
    }

    // Validate if the supplier_id is valid
    let supplier_exists = SUPPLIER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, supplier)| supplier.id == payload.supplier_id)
    });
    if !supplier_exists {
        return Err(Message::InvalidPayload("Supplier ID is invalid.".to_string()));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1).map(|_| current_value + 1)
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

    // Update supplier's items_supplied_ids
    SUPPLIER_STORAGE.with(|storage| {
        storage.borrow_mut().iter_mut().for_each(|(_, supplier)| {
            if supplier.id == payload.supplier_id {
                supplier.items_supplied_ids.push(payload.item_id.clone());
            }
        })
    });

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
#[ic_cdk
::query]
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

// Function to get an order by ID
#[ic_cdk::query]
fn get_order_by_id(order_id: String) -> Result<Order, Message> {
    ORDER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, order)| order.id == order_id)
            .map(|(_, order)| order.clone())
            .ok_or(Message::NotFound("Order not found".to_string()))
    })
}

// Function to update an item by ID
#[ic_cdk::update]
fn update_item(item_id: String, payload: ItemPayload) -> Result<Item, Message> {
    if payload.name.is_empty() || payload.description.is_empty() || payload.quantity == 0 {
        return Err(Message::InvalidPayload(
            "Ensure 'name', 'description', and 'quantity' are provided.".to_string(),
        ));
    }

    ITEM_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let item_key = storage.iter().find(|(_, item)| item.id == item_id).map(|(k, _)| k);
        match item_key {
            Some(key) => {
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

// Function to update a supplier by ID
#[ic_cdk::update]
fn update_supplier(supplier_id: String, payload: SupplierPayload) -> Result<Supplier, Message> {
    if payload.name.is_empty() || payload.contact_info.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'name' and 'contact_info' are provided.".to_string(),
        ));
    }

    SUPPLIER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let supplier_key = storage.iter().find(|(_, supplier)| supplier.id == supplier_id).map(|(k, _)| k);
        match supplier_key {
            Some(key) => {
                let updated_supplier = Supplier {
                    id: supplier_id.clone(),
                    name: payload.name,
                    contact_info: payload.contact_info,
                    items_supplied_ids: storage.get(&key).unwrap().items_supplied_ids.clone(),
                    created_at: time(),
                };
                storage.insert(key, updated_supplier.clone());
                Ok(updated_supplier)
            }
            None => Err(Message::NotFound("Supplier not found".to_string())),
        }
    })
}

// Function to delete an item by ID
#[ic_cdk::update]
fn delete_item(item_id: String) -> Result<Message, Message> {
    ITEM_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let item_key = storage.iter().find(|(_, item)| item.id == item_id).map(|(k, _)| k);
        match item_key {
            Some(key) => {
                storage.remove(&key);
                Ok(Message::Success("Item deleted".to_string()))
            }
            None => Err(Message::NotFound("Item not found".to_string())),
        }
    })
}

// Function to delete a supplier by ID
#[ic_cdk::update]
fn delete_supplier(supplier_id: String) -> Result<Message, Message> {
    SUPPLIER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let supplier_key = storage.iter().find(|(_, supplier)| supplier.id == supplier_id).map(|(k, _)| k);
        match supplier_key {
            Some(key) => {
                storage.remove(&key);
                Ok(Message::Success("Supplier deleted".to_string()))
            }
            None => Err(Message::NotFound("Supplier not found".to_string())),
        }
    })
}

// Function to delete an order by ID
#[ic_cdk::update]
fn delete_order(order_id: String) -> Result<Message, Message> {
    ORDER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let order_key = storage.iter().find(|(_, order)| order.id == order_id).map(|(k, _)| k);
        match order_key {
            Some(key) => {
                storage.remove(&key);
                Ok(Message::Success("Order deleted".to_string()))
            }
            None => Err(Message::NotFound("Order not found".to_string())),
        }
    })
}

// Export candid definitions for the canister
ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Encode;

    #[test]
    fn test_create_item() {
        let payload = ItemPayload {
            name: "Item1".to_string(),
            description: "Description1".to_string(),
            quantity: 10,
        };
        let result = create_item(payload);
        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.name, "Item1");
        assert_eq!(item.description, "Description1");
        assert_eq!(item.quantity, 10);
    }

    #[test]
    fn test_create_supplier() {
        let payload = SupplierPayload {
            name: "Supplier1".to_string(),
            contact_info: "Contact1".to_string(),
        };
        let result = create_supplier(payload);
        assert!(result.is_ok());
        let supplier = result.unwrap();
        assert_eq!(supplier.name, "Supplier1");
        assert_eq!(supplier.contact_info, "Contact1");
    }

    #[test]
    fn test_create_order() {
        let item_payload = ItemPayload {
            name: "Item1".to_string(),
            description: "Description1".to_string(),
            quantity: 10,
        };
        let item_result = create_item(item_payload);
        assert!(item_result.is_ok());
        let item = item_result.unwrap();

        let supplier_payload = SupplierPayload {
            name: "Supplier1".to_string(),
            contact_info: "Contact1".to_string(),
        };
        let supplier_result = create_supplier(supplier_payload);
        assert!(supplier_result.is_ok());
        let supplier = supplier_result.unwrap();

        let order_payload = OrderPayload {
            item_id: item.id.clone(),
            quantity: 5,
            supplier_id: supplier.id.clone(),
        };
        let result = create_order(order_payload);
        assert!(result.is_ok());
        let order = result.unwrap();
        assert_eq!(order.item_id, item.id);
        assert_eq!(order.quantity, 5);
        assert_eq!(order.supplier_id, supplier.id);
    }

    #[test]
    fn test_update_item() {
        let create_payload = ItemPayload {
            name: "Item1".to_string(),
            description: "Description1".to_string(),
            quantity: 10,
        };
        let create_result = create_item(create_payload);
        assert!(create_result.is_ok());
        let item = create_result.unwrap();

        let update_payload = ItemPayload {
            name: "UpdatedItem".to_string(),
            description: "UpdatedDescription".to_string(),
            quantity: 20,
        };
        let update_result = update_item(item.id.clone(), update_payload);
        assert!(update_result.is_ok());
        let updated_item = update_result.unwrap();
        assert_eq!(updated_item.name, "UpdatedItem");
        assert_eq!(updated_item.description, "UpdatedDescription");
        assert_eq!(updated_item.quantity, 20);
    }

    #[test]
    fn test_update_supplier() {
        let create_payload = SupplierPayload {
            name: "Supplier1".to_string(),
            contact_info: "Contact1".to_string(),
        };
        let create_result = create_supplier(create_payload);
        assert!(create_result.is_ok());
        let supplier = create_result.unwrap();

        let update_payload = SupplierPayload {
            name: "UpdatedSupplier".to_string(),
            contact_info: "UpdatedContact".to_string(),
        };
        let update_result = update_supplier(supplier.id.clone(), update_payload);
        assert!(update_result.is_ok());
        let updated_supplier = update_result.unwrap();
        assert_eq!(updated_supplier.name, "UpdatedSupplier");
        assert_eq!(updated_supplier.contact_info, "UpdatedContact");
    }

    #[test]
    fn test_delete_item() {
        let create_payload = ItemPayload {
            name: "Item1".to_string(),
            description: "Description1".to_string(),
            quantity: 10,
        };
        let create_result = create_item(create_payload);
        assert!(create_result.is_ok());
        let item = create_result.unwrap();

        let delete_result = delete_item(item.id.clone());
        assert!(delete_result.is_ok());

        let get_result = get_item_by_id(item.id);
        assert!(get_result.is_err());
    }

    #[test]
    fn test_delete_supplier() {
        let create_payload = SupplierPayload {
            name: "Supplier1".to_string(),
            contact_info: "Contact1".to_string(),
        };
        let create_result = create_supplier(create_payload);
        assert!(create_result.is_ok());
        let supplier = create_result.unwrap();

        let delete_result = delete_supplier(supplier.id.clone());
        assert!(delete_result.is_ok());

        let get_result = get_supplier_by_id(supplier.id);
        assert!(get_result.is_err());
    }

    #[test]
    fn test_delete_order() {
        let item_payload = ItemPayload {
            name: "Item1".to
            to_string(),
            description: "Description1".to_string(),
            quantity: 10,
        };
        let item_result = create_item(item_payload);
        assert!(item_result.is_ok());
        let item = item_result.unwrap();

        let supplier_payload = SupplierPayload {
            name: "Supplier1".to_string(),
            contact_info: "Contact1".to_string(),
        };
        let supplier_result = create_supplier(supplier_payload);
        assert!(supplier_result.is_ok());
        let supplier = supplier_result.unwrap();

        let order_payload = OrderPayload {
            item_id: item.id.clone(),
            quantity: 5,
            supplier_id: supplier.id.clone(),
        };
        let create_order_result = create_order(order_payload);
        assert!(create_order_result.is_ok());
        let order = create_order_result.unwrap();

        let delete_result = delete_order(order.id.clone());
        assert!(delete_result.is_ok());

        let get_result = get_order_by_id(order.id);
        assert!(get_result.is_err());
    }
}

// Main function to start the canister
#[ic_cdk::init]
fn init() {
    ic_cdk::setup();
}
