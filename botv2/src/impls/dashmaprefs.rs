/// Port of serenity !Send dashmap cache refs: https://github.com/serenity-rs/serenity/blob/current/src/cache/mod.rs#L58-L109

enum CacheRefInner<'a, K, V, T> {
    #[cfg(feature = "temp_cache")]
    Arc(Arc<V>),
    DashRef(Ref<'a, K, V, BuildHasher>),
    DashMappedRef(MappedRef<'a, K, T, V, BuildHasher>),
    ReadGuard(parking_lot::RwLockReadGuard<'a, V>),
}

pub struct CacheRef<'a, K, V, T = ()> {
    inner: CacheRefInner<'a, K, V, T>,
    phantom: std::marker::PhantomData<*const NotSend>,
}

impl<'a, K, V, T> CacheRef<'a, K, V, T> {
    fn new(inner: CacheRefInner<'a, K, V, T>) -> Self {
        Self {
            inner,
            phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "temp_cache")]
    fn from_arc(inner: MaybeOwnedArc<V>) -> Self {
        Self::new(CacheRefInner::Arc(inner.get_inner()))
    }

    fn from_ref(inner: Ref<'a, K, V, BuildHasher>) -> Self {
        Self::new(CacheRefInner::DashRef(inner))
    }

    fn from_mapped_ref(inner: MappedRef<'a, K, T, V, BuildHasher>) -> Self {
        Self::new(CacheRefInner::DashMappedRef(inner))
    }

    fn from_guard(inner: parking_lot::RwLockReadGuard<'a, V>) -> Self {
        Self::new(CacheRefInner::ReadGuard(inner))
    }
}

impl<K: Eq + Hash, V, T> std::ops::Deref for CacheRef<'_, K, V, T> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            #[cfg(feature = "temp_cache")]
            CacheRefInner::Arc(inner) => inner,
            CacheRefInner::DashRef(inner) => inner.value(),
            CacheRefInner::DashMappedRef(inner) => inner.value(),
            CacheRefInner::ReadGuard(inner) => inner,
        }
    }
}
