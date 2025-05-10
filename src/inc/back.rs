use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, Weak};

use slab::Slab;

pub struct Packet<T> {
    slab: Slab<T>,
    capacity_admin: bool,
    wc:bool,
    is_locked:bool,
}

impl<T> Packet<T> {
    fn new() -> Self {
        Self {
            slab: Slab::new(),
            capacity_admin: false,
            wc:false,
            is_locked:false,
        }
    }

    pub fn len(&self) -> usize {
        self.slab.len()
    }

    pub fn set_wval_data(&mut self, val: T) -> Result<usize,&str> {
        if self.wc && self.len() == self.capacity_size(){
            return Err("kapasiten yeterli değil");
        }else{
        let entry = self.slab.vacant_entry();
        let key = entry.key();
        entry.insert(val);
        Ok(key)
        }
    }

    pub fn getx_data_index(&self,val: T)-> Result<usize,&str> where T: PartialEq,{
        for index in 0..self.len() {
            if let Some(existing) = self.get_data(index) {
                if val == *existing {
                    return Ok(index);
                }
            }
        }
        return Err("data bulunamadı");
    }


    pub fn get_data(&self, key: usize) -> Option<&T> {
        Some(&self.slab[key])
    }

    pub fn capacity_size(&self) -> usize {
        self.slab.capacity()
    }

}

pub fn set_packet_large<T>(packet: &mut MutexGuard<'_, Packet<T>>, value: usize) -> bool {
    if value < packet.capacity_size() || !packet.capacity_admin {
        return false;
    }

    let madd_v = value - packet.capacity_size();
    packet.slab.reserve_exact(madd_v);
    if packet.capacity_size() == value {
        return true;
    }
    return false;
}

fn set_packet_large_ad<T>(packet: &mut MutexGuard<'_, Packet<T>>, value: usize){
    let madd_v = value - packet.capacity_size();
    packet.slab.reserve_exact(madd_v);
}

pub fn locker<'a, T>(locked: &'a Arc<Mutex<Packet<T>>>) -> Result<MutexGuard<'a, Packet<T>>, &'static str> {
    
    let mut l = locked.lock().unwrap();
    
    if l.is_locked {
        return Err("Bu Packet zaten kilitli durumda. Locker'ı unlock etmen lazım.");
    }

    l.is_locked = true;
    Ok(l)
}

pub fn unlocker<T>(mut guard: MutexGuard<'_, Packet<T>>) {
    guard.is_locked = false;
    drop(guard);
}

pub trait AllowedTypes: Send + Sync + 'static {}

#[macro_export]
macro_rules! add_type {
    ($($t:ty),*) => {
        $(impl AllowedTypes for $t {})*
    };
}

add_type!(
    i8, i16, i32,
    u8, u16, u32, u64,
    char, String, str 
);

static PACKET_MAP: OnceLock<Mutex<HashMap<TypeId, Weak<dyn Any + Send + Sync>>>> = OnceLock::new();
fn get_global_map() -> &'static Mutex<HashMap<TypeId, Weak<dyn Any + Send + Sync>>> {
    PACKET_MAP.get_or_init(|| Mutex::new(HashMap::new()))
}


pub struct Unset;
pub struct Set;

pub struct Distributor<State> {
    _state: PhantomData<State>,
}

impl Distributor<Unset> {
    pub fn new() -> Self {
        Distributor {
            _state: PhantomData,
        }
    }

    pub fn init<T: AllowedTypes>(
        &mut self,
        capacity_admin: bool,
        capacity: bool,
        capacity_size: usize,
    ) -> Option<Arc<Mutex<Packet<T>>>> {
        if !capacity {
            return None;
        }
        fn stc_wc<T>(packet: &Arc<Mutex<Packet<T>>>){
            packet.lock().unwrap().wc=true;
        }
        let type_id = TypeId::of::<T>();

        let map = get_global_map();
        let mut guard = map.lock().unwrap();

        if let Some(weak_any) = guard.get(&type_id) {
            if weak_any.upgrade().is_some() {
                panic!("Bu tip için zaten aktif bir paket oluşturuldu!");
            } else {
                guard.remove(&type_id);
            }
        }

        let packet = Arc::new(Mutex::new(Packet::<T>::new()));
        guard.insert(type_id, Arc::downgrade(&packet) as Weak<dyn Any + Send + Sync>);
        
        if capacity_admin {
            packet.lock().unwrap().capacity_admin = true;
        }

        if capacity {
            set_packet_large_ad(&mut packet.lock().unwrap(), capacity_size);
            stc_wc(&packet);
        }
        Some(packet)
    }
}

