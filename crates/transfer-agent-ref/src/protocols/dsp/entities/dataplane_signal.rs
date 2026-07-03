use common::dsp_common::data_address::DataAddress;

/// What the dataplane tells the manager over its channel. Each variant maps onto
/// a state transition.
#[derive(Debug)]
pub enum DataplaneSignal {
    Started(DataAddress),
    Completed,
    Terminated(String),
}
