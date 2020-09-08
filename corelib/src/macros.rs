/// 注册一个数据源
#[macro_export]
macro_rules! register_ds {
    ($namespace: ident) => {
        {
            Box::new($namespace::DataSourceImpl::new())
        }
    };
    ($namespace: ident : $($state: expr),*) => {
        {
            use crate::datasource::DataSource;
            let ds = Box::new($namespace::DataSourceImpl::new());
            $(
                $crate::register_state!(ds,$state.clone());
            )*
            ds
        }
    };
}

/// 注册一个实例到容器中
#[macro_export]
macro_rules! register_state {
    ($ds: expr, $state: expr) => {{
        $ds.get_register().set_state($state);
    }};
}

/// 创建一行结果集
#[macro_export]
macro_rules! row {
    ($($val: expr),*)=> {
        {
            let mut row = $crate::Row::new();
            $(
                row.push($val);
            )*
            row
        }
    };
}

/// 创建一个列定义
#[macro_export]
macro_rules! columns {
    [] => {{
        $crate::Columns::new()
    }};

    [$($type_d: ident : $name: expr),*] => {{
        let mut cols: $crate::Columns = $crate::Columns::new();
        $(
            cols.push($name, $crate::DataType::$type_d);
        )*

        cols
    }};

    [$($type_d: expr => $name: expr),*] => {{
        let mut cols: $crate::Columns = $crate::Columns::new();
        $(
            cols.push($name, $type_d);
        )*

        cols
    }};
}

/// 创建参数列表
#[macro_export]
macro_rules! args {
    ($($val: expr),*) => {{
        let mut args: $crate::Args = $crate::Args::new();
        $(
            args.push($val);
        )*

        args
    }};
}

/// 注册扩展函数
#[macro_export]
macro_rules! register_func {
    ($connect: expr, $namespace: ident) => {
        let name = $namespace::FunctionImpl::name();
        let args_size = $namespace::FunctionImpl::args();
        $connect.register_func(name, args_size, $namespace::FunctionImpl::invoke)?;
    };
}
