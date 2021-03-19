#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FrameType {
    Management,
    Control,
    Data,
    Unknown,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FrameSubType {
    AssoReq,
    AssoResp,
    ReassoReq,
    ReassoResp,
    ProbeReq,
    ProbeResp,
    Beacon,
    Atim,
    Disasso,
    Auth,
    Deauth,
    Data,
    DataCfAck,
    DataCfPull,
    DataCfAckCfPull,
    NullData,
    CfAck,
    CfPull,
    CfAckCfPull,
    QoS,
    QoSCfPull,
    QoSCfAckCfPull,
    QoSNullData,
    Reserved,
    UnHandled,
}
