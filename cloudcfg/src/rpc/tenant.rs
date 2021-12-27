#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingMsg {
    /// Who has pinged the server
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PongMsg {
    #[prost(enumeration = "pong_msg::Status", tag = "1")]
    pub status: i32,
}
/// Nested message and enum types in `PongMsg`.
pub mod pong_msg {
    /// Smallest possible success message. I don't think we need to set it
    /// to anything else than Success but Nice to have a way to tell a bit of a
    /// status
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Status {
        Success = 0,
        Error = 1,
        Maintenance = 2,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OperationRequest {
    /// id on which to perform the operation
    /// will be ignored for create but is mandatory on change operations
    /// set if your API endpoint needs it.
    #[prost(string, tag = "1")]
    pub uuid: ::prost::alloc::string::String,
    /// the serialized modification data to apply.
    #[prost(oneof = "operation_request::ObjectSchema", tags = "2, 3")]
    pub object_schema: ::core::option::Option<operation_request::ObjectSchema>,
}
/// Nested message and enum types in `OperationRequest`.
pub mod operation_request {
    /// the serialized modification data to apply.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ObjectSchema {
        #[prost(message, tag = "2")]
        Tenant(super::TenantOperationRequestSchema),
        #[prost(message, tag = "3")]
        User(super::UserOperationRequestSchema),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TenantOperationRequestSchema {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserOperationRequestSchema {
    /// tenant id
    #[prost(string, tag = "1")]
    pub tenant_id: ::prost::alloc::string::String,
    /// username
    #[prost(string, tag = "2")]
    pub username: ::prost::alloc::string::String,
    /// password of the new user
    #[prost(string, tag = "3")]
    pub password: ::prost::alloc::string::String,
    /// email for the user
    #[prost(string, tag = "4")]
    pub email: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OperationResponse {
    #[prost(enumeration = "operation_response::Status", tag = "1")]
    pub status: i32,
    /// an additional status message about the operation performed only set if there was an error
    /// optional
    #[prost(string, tag = "2")]
    pub response_message: ::prost::alloc::string::String,
    /// serialized bytes of the Result of the operation
    /// This allows to reduce API calls as the client can parse this field optionally if needed
    /// If one wants client control over this use the boolean return result object
    #[prost(oneof = "operation_response::Object", tags = "3, 4")]
    pub object: ::core::option::Option<operation_response::Object>,
}
/// Nested message and enum types in `OperationResponse`.
pub mod operation_response {
    /// enum describing the kind of return
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Status {
        Success = 0,
        Error = 1,
        Maintenance = 2,
    }
    /// serialized bytes of the Result of the operation
    /// This allows to reduce API calls as the client can parse this field optionally if needed
    /// If one wants client control over this use the boolean return result object
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Object {
        #[prost(message, tag = "3")]
        User(super::UserResponseSchema),
        #[prost(message, tag = "4")]
        Tenant(super::TenantResponseSchema),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserResponseSchema {
    /// tenant id
    #[prost(string, tag = "1")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub uuid: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub email: ::prost::alloc::string::String,
    #[prost(bool, tag = "5")]
    pub email_confirmed: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TenantResponseSchema {
    /// tenant id
    #[prost(string, tag = "1")]
    pub uuid: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListRequest {
    /// limit of amount of records to return
    #[prost(uint64, tag = "1")]
    pub limit: u64,
    /// offeset after which entry to return results
    #[prost(uint64, tag = "2")]
    pub offset: u64,
    /// filter to filter entities by
    #[prost(oneof = "list_request::Filter", tags = "3, 4")]
    pub filter: ::core::option::Option<list_request::Filter>,
}
/// Nested message and enum types in `ListRequest`.
pub mod list_request {
    /// filter to filter entities by
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Filter {
        #[prost(message, tag = "3")]
        User(super::UserFilter),
        #[prost(message, tag = "4")]
        Tenant(super::TenantFilter),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetRequest {
    /// filter to filter entities by
    #[prost(oneof = "get_request::Filter", tags = "3, 4")]
    pub filter: ::core::option::Option<get_request::Filter>,
}
/// Nested message and enum types in `GetRequest`.
pub mod get_request {
    /// filter to filter entities by
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Filter {
        #[prost(message, tag = "3")]
        User(super::UserFilter),
        #[prost(message, tag = "4")]
        Tenant(super::TenantFilter),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserFilter {
    /// tenant (this is mandatory)
    #[prost(string, tag = "1")]
    pub tenant_id: ::prost::alloc::string::String,
    /// email, either this or username needs to be set
    #[prost(string, tag = "2")]
    pub email: ::prost::alloc::string::String,
    /// username, either this or email needs to be set
    #[prost(string, tag = "3")]
    pub username: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TenantFilter {
    /// name of the tenant
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTenantResponse {
    #[prost(message, repeated, tag = "2")]
    pub tenants: ::prost::alloc::vec::Vec<TenantResponseSchema>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListUserResponse {
    #[prost(message, repeated, tag = "2")]
    pub users: ::prost::alloc::vec::Vec<UserResponseSchema>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoginRequest {
    #[prost(string, tag = "1")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub password: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoginResponse {
    /// The JWT token used for authentication
    #[prost(string, tag = "1")]
    pub auth_token: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub refresh_token: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicKeyResponse {
    #[prost(string, repeated, tag = "1")]
    pub public_key: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[doc = r" Generated client implementations."]
pub mod tenant_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct TenantClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TenantClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
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
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
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
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            TenantClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        #[doc = " A small rpc to ping to make sure we are connected, but also"]
        #[doc = " to help make a fast development function"]
        pub async fn ping(
            &mut self,
            request: impl tonic::IntoRequest<super::PingMsg>,
        ) -> Result<tonic::Response<super::PongMsg>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/Ping");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Tenant CRUD"]
        pub async fn list_tenants(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRequest>,
        ) -> Result<tonic::Response<super::ListTenantResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/ListTenants");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_tenant(
            &mut self,
            request: impl tonic::IntoRequest<super::GetRequest>,
        ) -> Result<tonic::Response<super::TenantResponseSchema>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            request: impl tonic::IntoRequest<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/CreateTenant");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_tenant(
            &mut self,
            request: impl tonic::IntoRequest<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/UpdateTenant");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_tenant(
            &mut self,
            request: impl tonic::IntoRequest<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/DeleteTenant");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " User CRUD"]
        pub async fn list_users(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRequest>,
        ) -> Result<tonic::Response<super::ListUserResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/ListUsers");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_user(
            &mut self,
            request: impl tonic::IntoRequest<super::GetRequest>,
        ) -> Result<tonic::Response<super::UserResponseSchema>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/GetUser");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_user(
            &mut self,
            request: impl tonic::IntoRequest<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/CreateUser");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_user(
            &mut self,
            request: impl tonic::IntoRequest<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/UpdateUser");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_user(
            &mut self,
            request: impl tonic::IntoRequest<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/DeleteUser");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = "Authentication API"]
        pub async fn login(
            &mut self,
            request: impl tonic::IntoRequest<super::LoginRequest>,
        ) -> Result<tonic::Response<super::LoginResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/Login");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_public_key(
            &mut self,
            request: impl tonic::IntoRequest<()>,
        ) -> Result<tonic::Response<super::PublicKeyResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tenant.Tenant/GetPublicKey");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod tenant_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with TenantServer."]
    #[async_trait]
    pub trait Tenant: Send + Sync + 'static {
        #[doc = " A small rpc to ping to make sure we are connected, but also"]
        #[doc = " to help make a fast development function"]
        async fn ping(
            &self,
            request: tonic::Request<super::PingMsg>,
        ) -> Result<tonic::Response<super::PongMsg>, tonic::Status>;
        #[doc = " Tenant CRUD"]
        async fn list_tenants(
            &self,
            request: tonic::Request<super::ListRequest>,
        ) -> Result<tonic::Response<super::ListTenantResponse>, tonic::Status>;
        async fn get_tenant(
            &self,
            request: tonic::Request<super::GetRequest>,
        ) -> Result<tonic::Response<super::TenantResponseSchema>, tonic::Status>;
        async fn create_tenant(
            &self,
            request: tonic::Request<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status>;
        async fn update_tenant(
            &self,
            request: tonic::Request<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status>;
        async fn delete_tenant(
            &self,
            request: tonic::Request<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status>;
        #[doc = " User CRUD"]
        async fn list_users(
            &self,
            request: tonic::Request<super::ListRequest>,
        ) -> Result<tonic::Response<super::ListUserResponse>, tonic::Status>;
        async fn get_user(
            &self,
            request: tonic::Request<super::GetRequest>,
        ) -> Result<tonic::Response<super::UserResponseSchema>, tonic::Status>;
        async fn create_user(
            &self,
            request: tonic::Request<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status>;
        async fn update_user(
            &self,
            request: tonic::Request<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status>;
        async fn delete_user(
            &self,
            request: tonic::Request<super::OperationRequest>,
        ) -> Result<tonic::Response<super::OperationResponse>, tonic::Status>;
        #[doc = "Authentication API"]
        async fn login(
            &self,
            request: tonic::Request<super::LoginRequest>,
        ) -> Result<tonic::Response<super::LoginResponse>, tonic::Status>;
        async fn get_public_key(
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
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
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
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/tenant.Tenant/Ping" => {
                    #[allow(non_camel_case_types)]
                    struct PingSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::PingMsg> for PingSvc<T> {
                        type Response = super::PongMsg;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
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
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
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
                    impl<T: Tenant> tonic::server::UnaryService<super::ListRequest> for ListTenantsSvc<T> {
                        type Response = super::ListTenantResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_tenants(request).await };
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
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
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
                    impl<T: Tenant> tonic::server::UnaryService<super::GetRequest> for GetTenantSvc<T> {
                        type Response = super::TenantResponseSchema;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetRequest>,
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
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
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
                    impl<T: Tenant> tonic::server::UnaryService<super::OperationRequest> for CreateTenantSvc<T> {
                        type Response = super::OperationResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OperationRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_tenant(request).await };
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
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/UpdateTenant" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateTenantSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::OperationRequest> for UpdateTenantSvc<T> {
                        type Response = super::OperationResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OperationRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_tenant(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateTenantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
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
                    impl<T: Tenant> tonic::server::UnaryService<super::OperationRequest> for DeleteTenantSvc<T> {
                        type Response = super::OperationResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OperationRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_tenant(request).await };
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
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/ListUsers" => {
                    #[allow(non_camel_case_types)]
                    struct ListUsersSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::ListRequest> for ListUsersSvc<T> {
                        type Response = super::ListUserResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_users(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListUsersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/GetUser" => {
                    #[allow(non_camel_case_types)]
                    struct GetUserSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::GetRequest> for GetUserSvc<T> {
                        type Response = super::UserResponseSchema;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_user(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/CreateUser" => {
                    #[allow(non_camel_case_types)]
                    struct CreateUserSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::OperationRequest> for CreateUserSvc<T> {
                        type Response = super::OperationResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OperationRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_user(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/UpdateUser" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateUserSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::OperationRequest> for UpdateUserSvc<T> {
                        type Response = super::OperationResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OperationRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_user(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/DeleteUser" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteUserSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::OperationRequest> for DeleteUserSvc<T> {
                        type Response = super::OperationResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OperationRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_user(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/Login" => {
                    #[allow(non_camel_case_types)]
                    struct LoginSvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<super::LoginRequest> for LoginSvc<T> {
                        type Response = super::LoginResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LoginRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).login(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LoginSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tenant.Tenant/GetPublicKey" => {
                    #[allow(non_camel_case_types)]
                    struct GetPublicKeySvc<T: Tenant>(pub Arc<T>);
                    impl<T: Tenant> tonic::server::UnaryService<()> for GetPublicKeySvc<T> {
                        type Response = super::PublicKeyResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<()>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_public_key(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPublicKeySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
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
