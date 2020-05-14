//! `Middleware` for conditionally enables another middleware.
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use futures::future::{ok, Either, FutureExt, LocalBoxFuture};
use std::sync::Arc;

pub struct Condition<T, F> {
    trans: Arc<T>,
    enable: F
}

impl<T, F> Condition<T, F> {
    pub fn new(enable: F, trans: T) -> Self {
        Self { trans: Arc::new(trans), enable }
    }
}

impl<S, T, F> Transform<S> for Condition<T, F>
    where
        S: Service + 'static,
        T: Transform<S, Request = S::Request, Response = S::Response, Error = S::Error> + 'static,
        T::Future: 'static,
        T::InitError: 'static,
        T::Transform: 'static,
        F : Fn(&S::Request) -> bool + Clone + 'static
{
    type Request = S::Request;
    type Response = S::Response;
    type Error = T::Error;
    type InitError = T::InitError;
    type Transform = ConditionMiddleware<T, S, F>;
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ConditionMiddleware {
            trans: self.trans.clone(),
            service: service,
            enable: self.enable.clone()
        })
            .boxed_local()
    }
}

pub struct ConditionMiddleware<T, S, F> {
    trans: Arc<T>,
    service: S,
    enable: F
}

impl<T, S, F> Service for ConditionMiddleware<T, S, F>
    where
        S: Service + 'static,
        F: Fn(&S::Request) -> bool
{
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;
    type Future = Either<S::Future, S::Future>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: S::Request) -> Self::Future {
        if (self.enable)(&req) {

        }
        unimplemented!()
        // use ConditionMiddleware::*;
        // match self {
        //     Enable(service) => Either::Left(service.call(req)),
        //     Disable(service) => Either::Right(service.call(req)),
        // }
    }
}