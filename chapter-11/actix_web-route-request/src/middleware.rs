use std::rc::Rc;
use actix_web::dev::{
    forward_ready,
    Transform,
    Service,
    ServiceRequest,
    ServiceResponse
};
use actix_web::{web, Result, Error};
use futures::future::LocalBoxFuture;
use futures::FutureExt;
use core::future::{ready, Ready};

use crate::counter::CountState;

pub struct CounterMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CounterMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let state = req.app_data::<web::Data<CountState>>().unwrap();
        state.update_count();
        let service = self.service.clone();
        async move { 
            let res = service.call(req).await?;
            Ok(res)
        }
        .boxed_local()
    }
}

pub struct Counter;

impl <S, B> Transform<S, ServiceRequest> for Counter 
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CounterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CounterMiddleware {
            service: Rc::new(service),
        }))
    }
} 
