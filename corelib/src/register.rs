use state::Container;

/// 注册器，用于把指定实例注册到容器中
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
        let _ = self.container.set::<_>(value);
    }

    pub fn get_state<T>(&self) -> T
    where
        T: Sync + Send + Clone + 'static,
    {
        self.container.get::<T>().clone()
    }
}

#[test]
fn test() {
    let register = Register::new();
    register.set_state(10);

    assert_eq!(10, register.get_state());
}
