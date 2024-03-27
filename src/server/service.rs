// SPDX-FileCopyrightText: Copyright (c) 2017-2024 slowtec GmbH <post@slowtec.de>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::ops::Deref;

use crate::{Exception, Response};

#[async_trait::async_trait]
/// A Modbus server service.
pub trait Service: Sync {
    /// Requests handled by the service.
    type Request: Send;

    /// Process the request and return the response asynchronously.
    async fn call(&self, req: Self::Request) -> Result<Response, Exception>;
}

#[async_trait::async_trait]
impl<D> Service for D
where
    D: Deref + ?Sized + Sync,
    D::Target: Service,
{
    type Request = <D::Target as Service>::Request;

    /// A forwarding blanket impl to support smart pointers around [`Service`].
    async fn call(&self, req: Self::Request) -> Result<Response, Exception> {
        self.deref().call(req).await
    }
}
