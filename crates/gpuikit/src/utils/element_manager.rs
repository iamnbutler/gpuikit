use gpui::{App, Global, SharedString};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Powers gpuikit's syntactic sugar, allowing elements to not
/// have to specify ids manually when created.
#[derive(Debug, Clone)]
pub struct ElementManager {
    next_id: Arc<AtomicUsize>,
}

impl ElementManager {
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get the next button ID, incrementing the counter
    pub fn id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    // Note: this is intentionally not pub
    // we don't want to expose the current ID publicly
    // as it could get used multiple times leading to collisions
    #[allow(dead_code)]
    fn current(&self) -> usize {
        self.next_id.load(Ordering::SeqCst)
    }
}

impl Default for ElementManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global wrapper for ElementManager to make it accessible throughout the app
#[derive(Debug, Clone)]
pub struct GlobalElementManager(pub Arc<ElementManager>);

impl GlobalElementManager {
    pub fn new() -> Self {
        Self(Arc::new(ElementManager::new()))
    }
}

impl Default for GlobalElementManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for GlobalElementManager {
    type Target = Arc<ElementManager>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for GlobalElementManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Global for GlobalElementManager {}

/// Extension trait for App to access the element manager
pub trait ElementManagerExt {
    /// Get a reference to the global element manager
    fn element_manager(&self) -> &Arc<ElementManager>;

    /// Get the next button ID
    fn next_id(&self) -> usize;

    /// Get the next id as a `gpui::ElementId::NamedInteger`
    fn next_id_named(&self, name: impl Into<SharedString>) -> gpui::ElementId;
}

impl ElementManagerExt for App {
    fn element_manager(&self) -> &Arc<ElementManager> {
        &self.global::<GlobalElementManager>().0
    }

    fn next_id(&self) -> usize {
        self.element_manager().id()
    }

    fn next_id_named(&self, name: impl Into<SharedString>) -> gpui::ElementId {
        let name = name.into();
        let id = self.next_id();

        gpui::ElementId::named_usize(name, id)
    }
}

pub fn init(cx: &mut App) {
    cx.set_global(GlobalElementManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_manager_next() {
        let manager = ElementManager::new();

        assert_eq!(manager.id(), 0);
        assert_eq!(manager.id(), 1);
        assert_eq!(manager.id(), 2);
    }

    #[test]
    fn test_element_manager_thread_safe() {
        let manager = Arc::new(ElementManager::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = std::thread::spawn(move || {
                for _ in 0..100 {
                    manager_clone.id();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should have incremented 1000 times (10 threads * 100 increments each)
        assert_eq!(manager.current(), 1000);
    }
}
