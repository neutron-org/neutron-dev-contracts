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

pub mod contractmanager {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct QueryAddressFailuresRequest {
        #[prost(string, tag = "1")]
        pub address: ::prost::alloc::string::String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct QueryFailuresRequest {
        #[prost(string, tag = "1")]
        pub address: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "2")]
        pub pagination:
            ::core::option::Option<::cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest>,
    }
}
