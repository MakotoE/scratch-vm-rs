use gloo_timers::future::TimeoutFuture;
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Debug)]
pub struct DebugController {
    semphore: Arc<ControllerSemaphore>,
    display_debug: tokio::sync::RwLock<bool>,
    interval_id: tokio::sync::RwLock<i32>,
}

impl DebugController {
    pub fn new() -> Self {
        Self {
            semphore: Arc::new(ControllerSemaphore::new()),
            display_debug: tokio::sync::RwLock::new(false),
            interval_id: tokio::sync::RwLock::new(0),
        }
    }

    pub async fn wait(&self) {
        self.semphore.acquire().await;
    }

    pub async fn continue_(&self, _speed: Speed) {
        web_sys::window()
            .unwrap()
            .clear_interval_with_handle(*self.interval_id.read().await);
        self.semphore.reset().await;
        self.semphore.set_blocking(false).await;
        *self.display_debug.write().await = false;

        log::info!("continuing");
    }

    pub async fn pause(&self) {
        web_sys::window()
            .unwrap()
            .clear_interval_with_handle(*self.interval_id.read().await);
        self.semphore.reset().await;
        self.semphore.set_blocking(true).await;
        *self.display_debug.write().await = true;

        log::info!("paused");
    }

    pub fn step(&self) {
        self.semphore.add_permit();

        log::info!("step");
    }

    pub async fn display_debug(&self) -> bool {
        *self.display_debug.read().await
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Speed {
    Normal,
}

#[derive(Debug)]
struct ControllerSemaphore {
    semaphore: tokio::sync::Semaphore,
    blocking: tokio::sync::RwLock<bool>,
    i: RefCell<u64>,
}

impl ControllerSemaphore {
    fn new() -> Self {
        Self {
            semaphore: tokio::sync::Semaphore::new(0),
            blocking: tokio::sync::RwLock::new(false),
            i: RefCell::new(0),
        }
    }

    async fn acquire(&self) {
        let mut i = self.i.borrow_mut();
        if *self.blocking.read().await {
            self.semaphore.acquire().await.forget();
        } else if *i % 0x1000 == 0 {
            TimeoutFuture::new(0).await;
        }

        *i += 1;
    }

    fn add_permit(&self) {
        self.semaphore.add_permits(1);
    }

    async fn set_blocking(&self, blocking: bool) {
        *self.blocking.write().await = blocking;

        if !blocking {
            self.add_permit();
        }
    }

    async fn reset(&self) {
        while self.semaphore.available_permits() > 0 {
            match self.semaphore.try_acquire() {
                Ok(p) => p.forget(),
                Err(_) => break,
            }
        }
        *self.blocking.write().await = false;
    }

    #[allow(dead_code)]
    async fn available(&self) -> bool {
        self.semaphore.available_permits() > 0 || !*self.blocking.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn controller_semaphore() {
        let semaphore = ControllerSemaphore::new();
        assert!(semaphore.available().await);
        semaphore.set_blocking(true).await;
        assert!(!semaphore.available().await);
        semaphore.add_permit();
        assert!(semaphore.available().await);
        semaphore.acquire().await;
        assert!(!semaphore.available().await);
        semaphore.reset().await;
        assert!(semaphore.available().await);
    }
}
