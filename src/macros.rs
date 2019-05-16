macro_rules! read_val {
    ( $( $x:expr ),* ) => {
        {
            $(
                $x.lock().borrow()
            )*
        }
    };
}

macro_rules! write_val {
    ( $( $x:expr ),* ) => {
        {
            $(
                $x.lock().borrow_mut()
            )*
        }
    };
}
