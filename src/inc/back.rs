use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::sync::{Arc, OnceLock, Weak};
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use parking_lot::{RwLock, RwLockWriteGuard};
use slab::Slab;

#[derive(Debug, PartialEq)]
pub enum PacketError {
    CapacityInsufficient,
    DataNotFound,
}

pub struct Packet<T> {
    slab: Slab<T>,
    capacity_admin: bool,
    wc: bool,
}

impl<T> Packet<T> {
    fn new() -> Self {
        Self {
            slab: Slab::new(),
            capacity_admin: false,
            wc: false,
        }
    }

    pub fn len(&self) -> usize {
        self.slab.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slab.is_empty()
    }

    // Yazma operasyonu: Kapasite sınırı (wc) aktifse kontrol eder
    pub fn set_wval_data(&mut self, val: T) -> Result<usize, PacketError> {
        if self.wc && self.len() >= self.capacity_size() {
            return Err(PacketError::CapacityInsufficient);
        }

        let entry = self.slab.vacant_entry();
        let key = entry.key();
        entry.insert(val);
        Ok(key)
    }

    // Okuma operasyonu: Paralel çalışır
    pub fn getx_data_index(&self, val: T) -> Result<usize, PacketError>
    where
        T: PartialEq,
    {
        self.slab
            .iter()
            .find(|(_, existing)| val == **existing)
            .map(|(index, _)| index)
            .ok_or(PacketError::DataNotFound)
    }

    pub fn get_data(&self, key: usize) -> Option<&T> {
        self.slab.get(key)
    }

    // YENİ: İndeks üzerinden güvenli veri silme ve bellek yuvasını boşaltma
    pub fn remove_data(&mut self, key: usize) -> Option<T> {
        if self.slab.contains(key) {
            Some(self.slab.remove(key))
        } else {
            None
        }
    }

    pub fn capacity_size(&self) -> usize {
        self.slab.capacity()
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.slab.reserve_exact(additional);
    }
}

pub fn set_packet_large<T>(packet: &mut RwLockWriteGuard<'_, Packet<T>>, value: usize) -> bool {
    if value < packet.capacity_size() || !packet.capacity_admin {
        return false;
    }
    let madd_v = value - packet.capacity_size();
    packet.reserve_exact(madd_v);
    packet.capacity_size() == value
}

// --- GLOBAL PACKET MAP ---

pub trait AllowedTypes: Send + Sync + 'static {}

#[macro_export]
macro_rules! add_type {
    ($($t:ty),*) => {
        $(impl AllowedTypes for $t {})*
    };
}

add_type!(i8, i16, i32, u8, u16, u32, u64, char, String);

static PACKET_MAP: OnceLock<DashMap<TypeId, Weak<dyn Any + Send + Sync>>> = OnceLock::new();

fn get_global_map() -> &'static DashMap<TypeId, Weak<dyn Any + Send + Sync>> {
    PACKET_MAP.get_or_init(DashMap::new)
}

// --- TYPESTATE DESENİ ---

pub struct Unset;
pub struct Set;

pub struct Distributor<State> {
    _state: PhantomData<State>,
}

impl Default for Distributor<Unset> {
    fn default() -> Self {
        Self::new()
    }
}

impl Distributor<Unset> {
    pub fn new() -> Self {
        Distributor {
            _state: PhantomData,
        }
    }

    pub fn init<T: AllowedTypes>(
        self,
        capacity_admin: bool,
        capacity: bool,
        capacity_size: usize,
    ) -> Result<(Distributor<Set>, Arc<RwLock<Packet<T>>>), &'static str> {
        let type_id = TypeId::of::<T>();
        let map = get_global_map();

        // İlgili TypeId hücresini atomik olarak kilitleyip kontrol ediyoruz
        match map.entry(type_id) {
            Entry::Occupied(mut occupied) => {
                // Eğer içerideki Weak referans hala canlıysa (drop edilmemişse) hata dön
                if occupied.get().upgrade().is_some() {
                    return Err("Bu tip için zaten aktif bir paket oluşturuldu!");
                }

                // Önceki havuz kapanmışsa (Weak ölmüşse), yenisini oluşturup üzerine yazdırmak için
                let packet = Self::create_packet::<T>(capacity_admin, capacity, capacity_size);
                occupied.insert(Arc::downgrade(&packet) as Weak<dyn Any + Send + Sync>);

                Ok((
                    Distributor {
                        _state: PhantomData,
                    },
                    packet,
                ))
            }
            Entry::Vacant(vacant) => {
                let packet = Self::create_packet::<T>(capacity_admin, capacity, capacity_size);
                vacant.insert(Arc::downgrade(&packet) as Weak<dyn Any + Send + Sync>);

                Ok((
                    Distributor {
                        _state: PhantomData,
                    },
                    packet,
                ))
            }
        }
    }

    fn create_packet<T: AllowedTypes>(
        capacity_admin: bool,
        capacity: bool,
        capacity_size: usize,
    ) -> Arc<RwLock<Packet<T>>> {
        let packet = Arc::new(RwLock::new(Packet::<T>::new()));
        {
            let mut packet_guard = packet.write();
            packet_guard.capacity_admin = capacity_admin;

            if capacity {
                let current_capacity = packet_guard.capacity_size();
                if capacity_size > current_capacity {
                    packet_guard.reserve_exact(capacity_size - current_capacity);
                }
                packet_guard.wc = true;
            }
        }
        packet
    }
}

impl Distributor<Set> {
    pub fn reset(self) -> Distributor<Unset> {
        Distributor {
            _state: PhantomData,
        }
    }
}
