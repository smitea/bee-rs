use state::Container;

pub struct Register {
    container: Container,
}

impl Register {
    pub fn new() -> Self {
        Self {
            container: Container::new(),
        }
    }

    pub fn set_state<T>(&self, value: T)
    where
        T: Sync + Send + Clone + 'static,
    {
        self.container.set(value);
    }

    pub fn get_state<T>(&self) -> T
    where
        T: Sync + Send + Clone + 'static,
    {
        self.container.get::<T>().clone()
    }
}
