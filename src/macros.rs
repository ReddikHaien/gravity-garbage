#[macro_export]
macro_rules! set_field {
    ($interface:ident, $object:ident . $field:ident = $value:expr) => {
        {
            let obj = $value;
            $interface.disconnect($object.clone_transfer(),$object.$field.clone_transfer());
            $interface.connect($object.clone_transfer(),obj.clone_transfer());
            $object.$field = obj.clone_unpinned();
        }
    };
}

