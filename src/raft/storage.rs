pub trait Storage {
    // TODO(tbg): split this into two interfaces, LogStorage and StateStorage

    // Init
    fn initial_state();
}