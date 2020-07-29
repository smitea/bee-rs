#[macro_export]
macro_rules! register_ds {
    ($namespace: ident) => {
        {
            Box::new($namespace::DataSourceImpl::new())
        }
    };
}

#[macro_export]
macro_rules! register_state {
    ($ds: expr, $state: expr) => {
        {
            $ds.get_register().set_state($state);
        }
    };
}

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