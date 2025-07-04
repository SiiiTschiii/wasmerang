// This file is @generated by prost-build.
/// Argument expected by set_envoy_filter_state in envoy
/// <https://github.com/envoyproxy/envoy/blob/d741713c376d1e024236519fb59406c05702ad77/source/extensions/common/wasm/foreign.cc#L116>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetEnvoyFilterStateArguments {
    #[prost(string, tag = "1")]
    pub path: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
    #[prost(enumeration = "LifeSpan", tag = "3")]
    pub span: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum LifeSpan {
    FilterChain = 0,
    DownstreamRequest = 1,
    DownstreamConnection = 2,
}
impl LifeSpan {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            LifeSpan::FilterChain => "FilterChain",
            LifeSpan::DownstreamRequest => "DownstreamRequest",
            LifeSpan::DownstreamConnection => "DownstreamConnection",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FilterChain" => Some(Self::FilterChain),
            "DownstreamRequest" => Some(Self::DownstreamRequest),
            "DownstreamConnection" => Some(Self::DownstreamConnection),
            _ => None,
        }
    }
}
