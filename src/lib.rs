//! `Middleware` for conditionally enables another middleware.
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use futures::future::{ok, Either, FutureExt, LocalBoxFuture};
use std::sync::Arc;
use crate::ServiceState::Verified;

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
            service: ServiceState::Unverified(Some(service)),
            enable: self.enable.clone(),
        })
            .boxed_local()
    }
}

enum ServiceState<U, V> {
    Unverified(Option<U>),
    Verified(V)
}

impl<U, V> ServiceState<U, V> {
    pub fn get_verified(&mut self) -> Option<&mut V> {
        match self {
            ServiceState::Unverified(_) => None,
            ServiceState::Verified(x) => Some(x),
        }
    }
}

pub struct ConditionMiddleware<T, S, F> {
    trans: Arc<T>,
    service: ServiceState<S, S>,
    enable: F,
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
        let maybe_service = &mut self.service;
        let service_to_overwrite = match maybe_service {
            ServiceState::Unverified(s) => {
                let service = s.take().unwrap();
                if (self.enable)(&req) {
                    Some(service)
                } else {
                    Some(service)
                }
            },
            Verified(_) => None,
        };
        if let Some(service_to_overwrite) = service_to_overwrite {
            self.service = Verified(service_to_overwrite)
        }
        let verified_service = self.service.get_verified().unwrap();
        unimplemented!()
        // use ConditionMiddleware::*;
        // match self {
        //     Enable(service) => Either::Left(service.call(req)),
        //     Disable(service) => Either::Right(service.call(req)),
        // }
    }
}