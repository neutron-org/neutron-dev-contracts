pub mod interchaintx {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct QueryParams {}
}

pub mod interchainqueries {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct QueryParams {}
}

pub mod feeburner {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct QueryParams {}

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct QueryTotalBurnedNeutronsAmountRequest {}
}
