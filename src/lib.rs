use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use futures::future::{ok, Either, FutureExt, LocalBoxFuture};
use std::sync::Arc;

pub struct Condition<T, F> {
    trans: Arc<T>,
    enable: F,
}

impl<T, F> Condition<T, F> {
    pub fn new(enable: F, trans: T) -> Self {
        Self { trans: Arc::new(trans), enable }
    }
}

impl<S, T, F> Transform<S> for Condition<T, F>
    where
        S: Service + 'static,
        T: Transform<S, Request = S::Request, Response = S::Response, Error = S::Error>,
        T::Future: 'static,
        T::InitError: 'static,
        T::Transform: 'static,
        <T as Transform<S>>::Transform: Transform<S>,
        F : Clone
{
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;
    type Transform = ConditionMiddleware<T::Transform, F, S>;
    type InitError = T::InitError;
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        Either::Right(ok(ConditionMiddleware {
            transform: self.trans.clone(),
            enable: self.enable.clone(),
            service: Some(service)
        }))
            .boxed_local()
    }
}

pub struct ConditionMiddleware<T, F, S> {
    transform: Arc<T>,
    enable: F,
    service: Option<S>
}

impl<T, F, S> Service for ConditionMiddleware<T, F, S>
    where
        T: Transform<S>,
        S: Service<Request = T::Request, Response = T::Response, Error = T::Error>,
{
    type Request = T::Request;
    type Response = T::Response;
    type Error = T::Error;
    type Future = Either<T::Future, S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: T::Request) -> Self::Future {
        if true {
            let service = self.service.take();
            self.transform.new_transform(service.unwrap())
        } else {
            self.service.unwrap().call()
        }
    }
}