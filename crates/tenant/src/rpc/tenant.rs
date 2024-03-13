#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AttributeRequest {
    #[prost(string, tag = "1")]
    pub principal: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub attribute: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "3")]
    pub value: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DefineRoleRequest {
    #[prost(string, tag = "1")]
    pub ident: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub permissions: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RoleRequest {
    #[prost(string, tag = "1")]
    pub ident: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub tenant: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub principal: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RevokeTokenRequest {
    #[prost(string, tag = "1")]
    pub principal_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub tenant_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RefreshTokenResponse {
    #[prost(string, tag = "1")]
    pub token: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPrincipalAuthResponse {
    #[prost(bool, tag = "1")]
    pub encrypted: bool,
    #[prost(string, tag = "2")]
    pub token: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPrincipalAuthRequest {
    #[prost(string, tag = "1")]
    pub principal_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(bool, tag = "3")]
    pub encrypted: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeletePrincipalRequest {
    #[prost(string, tag = "1")]
    pub principal_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub tenant_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemovePublicKeyRequest {
    #[prost(string, tag = "1")]
    pub principal_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub fingerprint: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddPublicKeyRequest {
    #[prost(string, tag = "1")]
    pub principal_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub public_key: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreatePrincipalRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "3")]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "4")]
    pub public_keys: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTenantRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTenantRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "3")]
    pub parent: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusResponse {
    #[prost(enumeration = "status_response::Status", tag = "1")]
    pub code: i32,
    #[prost(string, optional, tag = "2")]
    pub message: ::core::option::Option<::prost::alloc::string::String>,
}
/// Nested message and enum types in `StatusResponse`.
pub mod status_response {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Status {
        Ok = 0,
        Error = 1,
    }
    impl Status {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Status::Ok => "OK",
                Status::Error => "ERROR",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "OK" => Some(Self::Ok),
                "ERROR" => Some(Self::Error),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingMsg {
    /// Who has pinged the server
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PongMsg {
    #[prost(enumeration = "pong_msg::Authenticated", tag = "1")]
    pub auth_status: i32,
    #[prost(string, optional, tag = "2")]
    pub message: ::core::option::Option<::prost::alloc::string::String>,
}
/// Nested message and enum types in `PongMsg`.
pub mod pong_msg {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Authenticated {
        None = 0,
        Sucessfull = 1,
        Failed = 2,
    }
    impl Authenticated {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Authenticated::None => "NONE",
                Authenticated::Sucessfull => "SUCESSFULL",
                Authenticated::Failed => "FAILED",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "NONE" => Some(Self::None),
                "SUCESSFULL" => Some(Self::Sucessfull),
                "FAILED" => Some(Self::Failed),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTenantRequest {
    /// limit of amount of records to return
    #[prost(uint64, tag = "1")]
    pub limit: u64,
    /// offeset after which entry to return results
    #[prost(uint64, tag = "2")]
    pub offset: u64,
    /// filter to filter entities by
    #[prost(message, optional, tag = "3")]
    pub filter: ::core::option::Option<TenantFilter>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPrincipalRequest {
    /// limit of amount of records to return
    #[prost(uint64, tag = "1")]
    pub limit: u64,
    /// offeset after which entry to return results
    #[prost(uint64, tag = "2")]
    pub offset: u64,
    /// filter to filter entities by
    #[prost(message, optional, tag = "3")]
    pub filter: ::core::option::Option<PrincipalFilter>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTenantRequest {
    /// filter to filter entities by
    #[prost(message, optional, tag = "1")]
    pub filter: ::core::option::Option<TenantFilter>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPrincipalRequest {
    /// filter to filter entities by
    #[prost(message, optional, tag = "1")]
    pub filter: ::core::option::Option<PrincipalFilter>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrincipalFilter {
    /// tenant (this is mandatory)
    #[prost(string, tag = "1")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(oneof = "principal_filter::MailOrName", tags = "2, 3")]
    pub mail_or_name: ::core::option::Option<principal_filter::MailOrName>,
}
/// Nested message and enum types in `PrincipalFilter`.
pub mod principal_filter {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum MailOrName {
        #[prost(string, tag = "2")]
        Email(::prost::alloc::string::String),
        #[prost(string, tag = "3")]
        Name(::prost::alloc::string::String),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TenantFilter {
    /// name of the tenant
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTenantResponse {
    #[prost(message, repeated, tag = "1")]
    pub tenants: ::prost::alloc::vec::Vec<TenantResponse>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPrincipalResponse {
    #[prost(message, repeated, tag = "1")]
    pub principals: ::prost::alloc::vec::Vec<PrincipalResponse>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrincipalResponse {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "3")]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "4")]
    pub email_confirmed: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TenantResponse {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicKeyResponse {
    #[prost(string, repeated, tag = "1")]
    pub public_key: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Generated server implementations.
pub mod tenant_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with TenantServer.
    #[async_trait]
    pub trait Tenant: Send + Sync + 'static {
        /// A small rpc to ping to make sure we are connected, but also
        /// to help make a fast development function
        async fn ping(
            &self,
            request: tonic::Request<super::PingMsg>,
        ) -> std::result::Result<tonic::Response<super::PongMsg>, tonic::Status>;
        /// Tenant Public API
        async fn list_tenants(
            &self,
            request: tonic::Request<super::ListTenantRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTenantResponse>,
            tonic::Status,
        >;
        async fn get_tenant(
            &self,
            request: tonic::Request<super::GetTenantRequest>,
        ) -> std::result::Result<tonic::Response<super::TenantResponse>, tonic::Status>;
        async fn create_tenant(
            &self,
            request: tonic::Request<super::CreateTenantRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn delete_tenant(
            &self,
            request: tonic::Request<super::DeleteTenantRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        /// Principal Public API
        async fn list_principals(
            &self,
            request: tonic::Request<super::ListPrincipalRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListPrincipalResponse>,
            tonic::Status,
        >;
        async fn get_principal(
            &self,
            request: tonic::Request<super::GetPrincipalRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PrincipalResponse>,
            tonic::Status,
        >;
        async fn create_principal(
            &self,
            request: tonic::Request<super::CreatePrincipalRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PrincipalResponse>,
            tonic::Status,
        >;
        async fn get_principal_auth(
            &self,
            request: tonic::Request<super::GetPrincipalAuthRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetPrincipalAuthResponse>,
            tonic::Status,
        >;
        async fn refresh_token(
            &self,
            request: tonic::Request<()>,
        ) -> std::result::Result<
            tonic::Response<super::RefreshTokenResponse>,
            tonic::Status,
        >;
        async fn revoke_token(
            &self,
            request: tonic::Request<super::RevokeTokenRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn add_public_key_to_principal(
            &self,
            request: tonic::Request<super::AddPublicKeyRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn remove_public_key(
            &self,
            request: tonic::Request<super::RemovePublicKeyRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn delete_principal(
            &self,
            request: tonic::Request<super::DeletePrincipalRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn get_server_public_key(
            &self,
            request: tonic::Request<()>,
        ) -> std::result::Result<
            tonic::Response<super::PublicKeyResponse>,
            tonic::Status,
        >;
        /// Role Permissions API
        async fn define_role(
            &self,
            request: tonic::Request<super::DefineRoleRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn add_role(
            &self,
            request: tonic::Request<super::RoleRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn remove_role(
            &self,
            request: tonic::Request<super::RoleRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        /// Principal Attributes API
        async fn add_attribute(
            &self,
            request: tonic::Request<super::AttributeRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn remove_attribute(
            &self,
            request: tonic::Request<super::AttributeRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct TenantServer<T: Tenant> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Tenant> TenantServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for TenantServer<T>
    where
        T: Tenant,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/tenant.Tenant/Ping" => {
                    #[allow(non_camel_case_types)]
                    struct PingSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::PingMsg>
                    for PingSvc<T> {
                        type Response = super::PongMsg;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PingMsg>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::ping(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/ListTenants" => {
                    #[allow(non_camel_case_types)]
                    struct ListTenantsSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::ListTenantRequest>
                    for ListTenantsSvc<T> {
                        type Response = super::ListTenantResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListTenantRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::list_tenants(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListTenantsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/GetTenant" => {
                    #[allow(non_camel_case_types)]
                    struct GetTenantSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::GetTenantRequest>
                    for GetTenantSvc<T> {
                        type Response = super::TenantResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTenantRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::get_tenant(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetTenantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/CreateTenant" => {
                    #[allow(non_camel_case_types)]
                    struct CreateTenantSvc<T: Tenant>(pub Arc<T>);
                    impl<
                        T: Tenant,
                    > tonic::server::UnaryService<super::CreateTenantRequest>
                    for CreateTenantSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateTenantRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::create_tenant(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateTenantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/DeleteTenant" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteTenantSvc<T: Tenant>(pub Arc<T>);
                    impl<
                        T: Tenant,
                    > tonic::server::UnaryService<super::DeleteTenantRequest>
                    for DeleteTenantSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteTenantRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::delete_tenant(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteTenantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/ListPrincipals" => {
                    #[allow(non_camel_case_types)]
                    struct ListPrincipalsSvc<T: Tenant>(pub Arc<T>);
                    impl<
                        T: Tenant,
                    > tonic::server::UnaryService<super::ListPrincipalRequest>
                    for ListPrincipalsSvc<T> {
                        type Response = super::ListPrincipalResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListPrincipalRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::list_principals(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListPrincipalsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/GetPrincipal" => {
                    #[allow(non_camel_case_types)]
                    struct GetPrincipalSvc<T: Tenant>(pub Arc<T>);
                    impl<
                        T: Tenant,
                    > tonic::server::UnaryService<super::GetPrincipalRequest>
                    for GetPrincipalSvc<T> {
                        type Response = super::PrincipalResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetPrincipalRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::get_principal(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPrincipalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/CreatePrincipal" => {
                    #[allow(non_camel_case_types)]
                    struct CreatePrincipalSvc<T: Tenant>(pub Arc<T>);
                    impl<
                        T: Tenant,
                    > tonic::server::UnaryService<super::CreatePrincipalRequest>
                    for CreatePrincipalSvc<T> {
                        type Response = super::PrincipalResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreatePrincipalRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::create_principal(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreatePrincipalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/GetPrincipalAuth" => {
                    #[allow(non_camel_case_types)]
                    struct GetPrincipalAuthSvc<T: Tenant>(pub Arc<T>);
                    impl<
                        T: Tenant,
                    > tonic::server::UnaryService<super::GetPrincipalAuthRequest>
                    for GetPrincipalAuthSvc<T> {
                        type Response = super::GetPrincipalAuthResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetPrincipalAuthRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::get_principal_auth(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPrincipalAuthSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/RefreshToken" => {
                    #[allow(non_camel_case_types)]
                    struct RefreshTokenSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<()>
                    for RefreshTokenSvc<T> {
                        type Response = super::RefreshTokenResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(&mut self, request: tonic::Request<()>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::refresh_token(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RefreshTokenSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/RevokeToken" => {
                    #[allow(non_camel_case_types)]
                    struct RevokeTokenSvc<T: Tenant>(pub Arc<T>);
                    impl<
                        T: Tenant,
                    > tonic::server::UnaryService<super::RevokeTokenRequest>
                    for RevokeTokenSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RevokeTokenRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::revoke_token(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RevokeTokenSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/AddPublicKeyToPrincipal" => {
                    #[allow(non_camel_case_types)]
                    struct AddPublicKeyToPrincipalSvc<T: Tenant>(pub Arc<T>);
                    impl<
                        T: Tenant,
                    > tonic::server::UnaryService<super::AddPublicKeyRequest>
                    for AddPublicKeyToPrincipalSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AddPublicKeyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::add_public_key_to_principal(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddPublicKeyToPrincipalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/RemovePublicKey" => {
                    #[allow(non_camel_case_types)]
                    struct RemovePublicKeySvc<T: Tenant>(pub Arc<T>);
                    impl<
                        T: Tenant,
                    > tonic::server::UnaryService<super::RemovePublicKeyRequest>
                    for RemovePublicKeySvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RemovePublicKeyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::remove_public_key(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RemovePublicKeySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/DeletePrincipal" => {
                    #[allow(non_camel_case_types)]
                    struct DeletePrincipalSvc<T: Tenant>(pub Arc<T>);
                    impl<
                        T: Tenant,
                    > tonic::server::UnaryService<super::DeletePrincipalRequest>
                    for DeletePrincipalSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeletePrincipalRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::delete_principal(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeletePrincipalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/GetServerPublicKey" => {
                    #[allow(non_camel_case_types)]
                    struct GetServerPublicKeySvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<()>
                    for GetServerPublicKeySvc<T> {
                        type Response = super::PublicKeyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(&mut self, request: tonic::Request<()>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::get_server_public_key(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetServerPublicKeySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/DefineRole" => {
                    #[allow(non_camel_case_types)]
                    struct DefineRoleSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::DefineRoleRequest>
                    for DefineRoleSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DefineRoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::define_role(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DefineRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/AddRole" => {
                    #[allow(non_camel_case_types)]
                    struct AddRoleSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::RoleRequest>
                    for AddRoleSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::add_role(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/RemoveRole" => {
                    #[allow(non_camel_case_types)]
                    struct RemoveRoleSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::RoleRequest>
                    for RemoveRoleSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::remove_role(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RemoveRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/AddAttribute" => {
                    #[allow(non_camel_case_types)]
                    struct AddAttributeSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::AttributeRequest>
                    for AddAttributeSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AttributeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::add_attribute(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddAttributeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/RemoveAttribute" => {
                    #[allow(non_camel_case_types)]
                    struct RemoveAttributeSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::AttributeRequest>
                    for RemoveAttributeSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AttributeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Tenant>::remove_attribute(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RemoveAttributeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Tenant> Clone for TenantServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Tenant> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Tenant> tonic::server::NamedService for TenantServer<T> {
        const NAME: &'static str = "tenant.Tenant";
    }
}
