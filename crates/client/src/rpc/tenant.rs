#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeletePrincipalRequest {
    #[prost(string, tag="1")]
    pub principal_name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub tenant_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemovePublicKeyRequest {
    #[prost(string, tag="1")]
    pub principal_name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub fingerprint: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddPublicKeyRequest {
    #[prost(string, tag="1")]
    pub principal_name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub public_key: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreatePrincipalRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(string, optional, tag="3")]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag="4")]
    pub public_keys: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTenantRequest {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTenantRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusResponse {
    #[prost(enumeration="status_response::Status", tag="1")]
    pub code: i32,
    #[prost(string, optional, tag="2")]
    pub message: ::core::option::Option<::prost::alloc::string::String>,
}
/// Nested message and enum types in `StatusResponse`.
pub mod status_response {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Status {
        Ok = 0,
        Error = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingMsg {
    /// Who has pinged the server
    #[prost(string, tag="1")]
    pub sender: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PongMsg {
    #[prost(string, tag="1")]
    pub pong: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTenantRequest {
    /// limit of amount of records to return
    #[prost(uint64, tag="1")]
    pub limit: u64,
    /// offeset after which entry to return results
    #[prost(uint64, tag="2")]
    pub offset: u64,
    /// filter to filter entities by
    #[prost(message, optional, tag="3")]
    pub filter: ::core::option::Option<TenantFilter>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPrincipalRequest {
    /// limit of amount of records to return
    #[prost(uint64, tag="1")]
    pub limit: u64,
    /// offeset after which entry to return results
    #[prost(uint64, tag="2")]
    pub offset: u64,
    /// filter to filter entities by
    #[prost(message, optional, tag="3")]
    pub filter: ::core::option::Option<PrincipalFilter>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTenantRequest {
    /// filter to filter entities by
    #[prost(message, optional, tag="1")]
    pub filter: ::core::option::Option<TenantFilter>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPrincipalRequest {
    /// filter to filter entities by
    #[prost(message, optional, tag="1")]
    pub filter: ::core::option::Option<PrincipalFilter>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrincipalFilter {
    /// tenant (this is mandatory)
    #[prost(string, tag="1")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(oneof="principal_filter::MailOrName", tags="2, 3")]
    pub mail_or_name: ::core::option::Option<principal_filter::MailOrName>,
}
/// Nested message and enum types in `PrincipalFilter`.
pub mod principal_filter {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum MailOrName {
        #[prost(string, tag="2")]
        Email(::prost::alloc::string::String),
        #[prost(string, tag="3")]
        Name(::prost::alloc::string::String),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TenantFilter {
    /// name of the tenant
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTenantResponse {
    #[prost(message, repeated, tag="1")]
    pub tenants: ::prost::alloc::vec::Vec<TenantResponse>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPrincipalResponse {
    #[prost(message, repeated, tag="1")]
    pub principals: ::prost::alloc::vec::Vec<PrincipalResponse>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrincipalResponse {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag="3")]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag="4")]
    pub email_confirmed: ::core::option::Option<bool>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TenantResponse {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicKeyResponse {
    #[prost(string, repeated, tag="1")]
    pub public_key: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Generated client implementations.
pub mod tenant_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct TenantClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TenantClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> TenantClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> TenantClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            TenantClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        /// A small rpc to ping to make sure we are connected, but also
        /// to help make a fast development function
        pub async fn ping(
            &mut self,
            request: impl tonic::IntoRequest<super::PingMsg>,
        ) -> Result<tonic::Response<super::PongMsg>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/Ping");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Tenant Public API
        pub async fn list_tenants(
            &mut self,
            request: impl tonic::IntoRequest<super::ListTenantRequest>,
        ) -> Result<tonic::Response<super::ListTenantResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tenant.Tenant/ListTenants",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_tenant(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTenantRequest>,
        ) -> Result<tonic::Response<super::TenantResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/GetTenant");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_tenant(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateTenantRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tenant.Tenant/CreateTenant",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_tenant(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteTenantRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tenant.Tenant/DeleteTenant",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Principal Public API
        pub async fn list_principals(
            &mut self,
            request: impl tonic::IntoRequest<super::ListPrincipalRequest>,
        ) -> Result<tonic::Response<super::ListPrincipalResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tenant.Tenant/ListPrincipals",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_principal(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPrincipalRequest>,
        ) -> Result<tonic::Response<super::PrincipalResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tenant.Tenant/GetPrincipal",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_principal(
            &mut self,
            request: impl tonic::IntoRequest<super::CreatePrincipalRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tenant.Tenant/CreatePrincipal",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn add_public_key_to_principal(
            &mut self,
            request: impl tonic::IntoRequest<super::AddPublicKeyRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tenant.Tenant/AddPublicKeyToPrincipal",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn remove_public_key(
            &mut self,
            request: impl tonic::IntoRequest<super::RemovePublicKeyRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tenant.Tenant/RemovePublicKey",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_principal(
            &mut self,
            request: impl tonic::IntoRequest<super::DeletePrincipalRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tenant.Tenant/DeletePrincipal",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_server_public_key(
            &mut self,
            request: impl tonic::IntoRequest<()>,
        ) -> Result<tonic::Response<super::PublicKeyResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tenant.Tenant/GetServerPublicKey",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod tenant_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with TenantServer.
    #[async_trait]
    pub trait Tenant: Send + Sync + 'static {
        /// A small rpc to ping to make sure we are connected, but also
        /// to help make a fast development function
        async fn ping(
            &self,
            request: tonic::Request<super::PingMsg>,
        ) -> Result<tonic::Response<super::PongMsg>, tonic::Status>;
        /// Tenant Public API
        async fn list_tenants(
            &self,
            request: tonic::Request<super::ListTenantRequest>,
        ) -> Result<tonic::Response<super::ListTenantResponse>, tonic::Status>;
        async fn get_tenant(
            &self,
            request: tonic::Request<super::GetTenantRequest>,
        ) -> Result<tonic::Response<super::TenantResponse>, tonic::Status>;
        async fn create_tenant(
            &self,
            request: tonic::Request<super::CreateTenantRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn delete_tenant(
            &self,
            request: tonic::Request<super::DeleteTenantRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        /// Principal Public API
        async fn list_principals(
            &self,
            request: tonic::Request<super::ListPrincipalRequest>,
        ) -> Result<tonic::Response<super::ListPrincipalResponse>, tonic::Status>;
        async fn get_principal(
            &self,
            request: tonic::Request<super::GetPrincipalRequest>,
        ) -> Result<tonic::Response<super::PrincipalResponse>, tonic::Status>;
        async fn create_principal(
            &self,
            request: tonic::Request<super::CreatePrincipalRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn add_public_key_to_principal(
            &self,
            request: tonic::Request<super::AddPublicKeyRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn remove_public_key(
            &self,
            request: tonic::Request<super::RemovePublicKeyRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn delete_principal(
            &self,
            request: tonic::Request<super::DeletePrincipalRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn get_server_public_key(
            &self,
            request: tonic::Request<()>,
        ) -> Result<tonic::Response<super::PublicKeyResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct TenantServer<T: Tenant> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
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
        ) -> Poll<Result<(), Self::Error>> {
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
                            let inner = self.0.clone();
                            let fut = async move { (*inner).ping(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_tenants(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListTenantsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_tenant(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetTenantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_tenant(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateTenantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).delete_tenant(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteTenantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_principals(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListPrincipalsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_principal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPrincipalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreatePrincipalRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_principal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreatePrincipalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).add_public_key_to_principal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddPublicKeyToPrincipalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).remove_public_key(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RemovePublicKeySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).delete_principal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeletePrincipalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_server_public_key(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetServerPublicKeySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
            }
        }
    }
    impl<T: Tenant> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Tenant> tonic::transport::NamedService for TenantServer<T> {
        const NAME: &'static str = "tenant.Tenant";
    }
}
