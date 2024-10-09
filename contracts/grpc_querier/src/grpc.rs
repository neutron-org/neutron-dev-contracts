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
            ::core::option::Option<::neutron_std::types::cosmos::base::query::v1beta1::PageRequest>,
    }
}
