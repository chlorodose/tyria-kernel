use core::{
    cell::UnsafeCell,
    fmt::Debug,
    hint::spin_loop,
    mem::MaybeUninit,
    ops::{Add, Neg, Sub},
    sync::atomic::{AtomicI32, AtomicU8, Ordering, fence},
};

use lock_api::{
    GuardSend, RawRwLock, RawRwLockDowngrade, RawRwLockUpgrade, RawRwLockUpgradeDowngrade,
};

use crate::utils::DeferGuard;

/// An spin lock supports upgradable share locks and downgrade write lock.
#[derive(Debug, Default)]
pub struct RawSpinLock(AtomicI32);
// NOTE: 0 => no locks, MIN => write lock, +n => n share locks, -n => n share locks(include 1 upgradeable)
// NOTE: when upgradeable is holding, no more shared locks could be created

unsafe impl RawRwLock for RawSpinLock {
    const INIT: Self = Self(AtomicI32::new(0));
    type GuardMarker = GuardSend;
    fn is_locked(&self) -> bool {
        self.0.load(Ordering::Relaxed) != 0
    }
    fn is_locked_exclusive(&self) -> bool {
        self.0.load(Ordering::Relaxed) == i32::MIN
    }
    fn try_lock_exclusive(&self) -> bool {
        self.0
            .compare_exchange(0, i32::MIN, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }
    fn lock_exclusive(&self) {
        if self.try_lock_exclusive() {
            return;
        }
        self.lock_upgradable();
        unsafe { self.upgrade() };
    }
    unsafe fn unlock_exclusive(&self) {
        debug_assert_eq!(
            self.0.load(Ordering::Relaxed),
            i32::MIN,
            "try unlock when not holding locks"
        );
        self.0.store(0, Ordering::Release);
    }
    fn try_lock_shared(&self) -> bool {
        let val = self.0.load(Ordering::Relaxed);
        if val.is_negative() {
            return false;
        }
        let new = val.checked_add(1).expect("lock share instance overflows");
        self.0
            .compare_exchange(val, new, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }
    fn lock_shared(&self) {
        while self
            .0
            .fetch_update(Ordering::Acquire, Ordering::Relaxed, |x| {
                if x.is_negative() {
                    return None;
                }
                Some(x.checked_add(1).expect("lock share instance overflows"))
            })
            .is_err()
        {
            spin_loop();
        }
    }
    unsafe fn unlock_shared(&self) {
        self.0
            .fetch_update(Ordering::Release, Ordering::Relaxed, |x| {
                debug_assert!(
                    ![0, -1, i32::MIN].contains(&x),
                    "try unlock when not holding locks"
                );
                match x.signum() {
                    1 => Some(x.sub(1)),
                    -1 => Some(x.add(1)),
                    _ => unreachable!(),
                }
            })
            .unwrap();
    }
}
unsafe impl RawRwLockDowngrade for RawSpinLock {
    unsafe fn downgrade(&self) {
        debug_assert_eq!(
            self.0.load(Ordering::Relaxed),
            i32::MIN,
            "try unlock when not holding locks"
        );
        self.0.store(1, Ordering::Release);
    }
}
unsafe impl RawRwLockUpgrade for RawSpinLock {
    fn try_lock_upgradable(&self) -> bool {
        let val = self.0.load(Ordering::Relaxed);
        if val.is_negative() {
            return false;
        }
        let new = val
            .checked_add(1)
            .expect("lock share instance overflows")
            .neg();
        self.0
            .compare_exchange(val, new, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }
    fn lock_upgradable(&self) {
        while self
            .0
            .fetch_update(Ordering::Acquire, Ordering::Relaxed, |x| {
                if x.is_negative() {
                    return None;
                }
                Some(
                    x.checked_add(1)
                        .expect("lock share instance overflows")
                        .neg(),
                )
            })
            .is_err()
        {
            spin_loop();
        }
    }
    unsafe fn try_upgrade(&self) -> bool {
        self.0
            .compare_exchange(-1, i32::MIN, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }
    unsafe fn upgrade(&self) {
        while self
            .0
            .compare_exchange_weak(-1, i32::MIN, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            spin_loop();
        }
    }
    unsafe fn unlock_upgradable(&self) {
        self.0
            .fetch_update(Ordering::Release, Ordering::Relaxed, |x| {
                debug_assert!(
                    x.is_negative() && x != i32::MIN,
                    "try unlock when not holding locks"
                );
                Some(x.add(1).neg())
            })
            .unwrap();
    }
}
unsafe impl RawRwLockUpgradeDowngrade for RawSpinLock {
    unsafe fn downgrade_to_upgradable(&self) {
        debug_assert_eq!(
            self.0.load(Ordering::Relaxed),
            i32::MIN,
            "try unlock when not holding locks"
        );
        self.0.store(-1, Ordering::Release);
    }
    unsafe fn downgrade_upgradable(&self) {
        self.0
            .fetch_update(Ordering::Release, Ordering::Relaxed, |x| {
                debug_assert!(
                    x.is_negative() && x != i32::MIN,
                    "try unlock when not holding locks"
                );
                Some(x.neg())
            })
            .unwrap();
    }
}

/// A synchronization primitive which can nominally be written to only once, using spin lock.
pub struct SpinOnceLock<T> {
    state: AtomicU8, // 0 => uninit, MAX => inited, _ => initing
    value: UnsafeCell<MaybeUninit<T>>,
}
unsafe impl<T: Send> Send for SpinOnceLock<T> {}
unsafe impl<T: Send + Sync> Sync for SpinOnceLock<T> {}
impl<T: Debug> Debug for SpinOnceLock<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("SpinOnceLock").field(&self.get()).finish()
    }
}
impl<T: Clone> Clone for SpinOnceLock<T> {
    fn clone(&self) -> Self {
        if let Some(val) = self.get() {
            Self {
                state: AtomicU8::new(u8::MAX),
                value: UnsafeCell::new(MaybeUninit::new(val.clone())),
            }
        } else {
            Self::new()
        }
    }
}
impl<T> Default for SpinOnceLock<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> SpinOnceLock<T> {
    /// Creates a new uninitialized `SpinOnceLock`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(0),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
    fn is_initialized(&self) -> bool {
        self.state.load(Ordering::Acquire) == u8::MAX
    }
    unsafe fn get_unchecked(&self) -> &T {
        unsafe { self.value.as_ref_unchecked().assume_init_ref() }
    }
    /// Get value if it's already initialized.
    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }
    /// Get value if it's already initialized, else create it using an closure.
    pub fn get_or_init(&self, init: impl FnOnce() -> T) -> &T {
        if !self.is_initialized() {
            loop {
                match self
                    .state
                    .compare_exchange_weak(0, 1, Ordering::Relaxed, Ordering::Relaxed)
                {
                    Ok(_) => {
                        let mut guard = DeferGuard::new((), |()| {
                            self.state.store(0, Ordering::Relaxed); // Clean state if init() panic
                        });
                        let val = init();
                        guard.forget();
                        unsafe { self.value.as_mut_unchecked().write(val) };
                        self.state.store(u8::MAX, Ordering::Release);
                        break;
                    }
                    Err(u8::MAX) => {
                        fence(Ordering::Acquire);
                        break;
                    }
                    _ => spin_loop(),
                }
            }
        }
        unsafe { self.get_unchecked() }
    }
}
impl<T> Drop for SpinOnceLock<T> {
    fn drop(&mut self) {
        if *self.state.get_mut() == u8::MAX {
            unsafe { self.value.get_mut().assume_init_drop() }
        }
    }
}
