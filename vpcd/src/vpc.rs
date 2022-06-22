#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetVpcRequest {
    #[prost(string, tag="1")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(bool, optional, tag="2")]
    pub get_dedault: ::core::option::Option<bool>,
    #[prost(string, optional, tag="3")]
    pub id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="4")]
    pub cidr: ::core::option::Option<::prost::alloc::string::String>,
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
pub struct CreateVpcRequest {
    #[prost(string, tag="1")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(bool, tag="3")]
    pub is_tenant_default: bool,
    #[prost(enumeration="create_vpc_request::VpcType", tag="4")]
    pub vpc_type: i32,
    #[prost(string, tag="5")]
    pub ip_pool_cidr: ::prost::alloc::string::String,
}
/// Nested message and enum types in `CreateVPCRequest`.
pub mod create_vpc_request {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum VpcType {
        Etherstub = 0,
        DistributedEtherstub = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListVpcRequest {
    #[prost(oneof="list_vpc_request::Tenant", tags="1, 2")]
    pub tenant: ::core::option::Option<list_vpc_request::Tenant>,
}
/// Nested message and enum types in `ListVPCRequest`.
pub mod list_vpc_request {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Tenant {
        #[prost(string, tag="1")]
        TenantId(::prost::alloc::string::String),
        #[prost(string, tag="2")]
        TenantName(::prost::alloc::string::String),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListVpcResponse {
    #[prost(string, tag="1")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub tenant_name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="3")]
    pub vpcs: ::prost::alloc::vec::Vec<VpcSchema>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VpcSchema {
    #[prost(string, tag="1")]
    pub tenant_id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub id: ::prost::alloc::string::String,
    #[prost(bool, tag="3")]
    pub is_tenant_default: bool,
    #[prost(enumeration="vpc_schema::VpcType", tag="4")]
    pub vpc_type: i32,
    #[prost(string, tag="5")]
    pub ip_pool_cidr: ::prost::alloc::string::String,
}
/// Nested message and enum types in `VPCSchema`.
pub mod vpc_schema {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum VpcType {
        Etherstub = 0,
        DistributedEtherstub = 1,
    }
}
/// Generated client implementations.
pub mod vpc_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct VpcClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl VpcClient<tonic::transport::Channel> {
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
    impl<T> VpcClient<T>
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
        ) -> VpcClient<InterceptedService<T, F>>
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
            VpcClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn list_vpcs(
            &mut self,
            request: impl tonic::IntoRequest<super::ListVpcRequest>,
        ) -> Result<tonic::Response<super::ListVpcResponse>, tonic::Status> {
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
            let path = http::uri::PathAndQuery::from_static("/vpc.VPC/ListVPCS");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_vpc(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateVpcRequest>,
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
            let path = http::uri::PathAndQuery::from_static("/vpc.VPC/CreateVPC");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_vpc(
            &mut self,
            request: impl tonic::IntoRequest<super::GetVpcRequest>,
        ) -> Result<tonic::Response<super::VpcSchema>, tonic::Status> {
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
            let path = http::uri::PathAndQuery::from_static("/vpc.VPC/GetVPC");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod vpc_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with VpcServer.
    #[async_trait]
    pub trait Vpc: Send + Sync + 'static {
        async fn list_vpcs(
            &self,
            request: tonic::Request<super::ListVpcRequest>,
        ) -> Result<tonic::Response<super::ListVpcResponse>, tonic::Status>;
        async fn create_vpc(
            &self,
            request: tonic::Request<super::CreateVpcRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        async fn get_vpc(
            &self,
            request: tonic::Request<super::GetVpcRequest>,
        ) -> Result<tonic::Response<super::VpcSchema>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct VpcServer<T: Vpc> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Vpc> VpcServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for VpcServer<T>
    where
        T: Vpc,
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
                "/vpc.VPC/ListVPCS" => {
                    #[allow(non_camel_case_types)]
                    struct ListVPCSSvc<T: Vpc>(pub Arc<T>);
                    impl<T: Vpc> tonic::server::UnaryService<super::ListVpcRequest>
                    for ListVPCSSvc<T> {
                        type Response = super::ListVpcResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListVpcRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_vpcs(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListVPCSSvc(inner);
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
                "/vpc.VPC/CreateVPC" => {
                    #[allow(non_camel_case_types)]
                    struct CreateVPCSvc<T: Vpc>(pub Arc<T>);
                    impl<T: Vpc> tonic::server::UnaryService<super::CreateVpcRequest>
                    for CreateVPCSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateVpcRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_vpc(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateVPCSvc(inner);
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
                "/vpc.VPC/GetVPC" => {
                    #[allow(non_camel_case_types)]
                    struct GetVPCSvc<T: Vpc>(pub Arc<T>);
                    impl<T: Vpc> tonic::server::UnaryService<super::GetVpcRequest>
                    for GetVPCSvc<T> {
                        type Response = super::VpcSchema;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetVpcRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_vpc(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetVPCSvc(inner);
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
    impl<T: Vpc> Clone for VpcServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Vpc> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Vpc> tonic::transport::NamedService for VpcServer<T> {
        const NAME: &'static str = "vpc.VPC";
    }
}
