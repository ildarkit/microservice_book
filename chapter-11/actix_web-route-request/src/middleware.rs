use std::cell::RefCell;
use std::rc::Rc;
use actix_web::dev::{
    forward_ready,
    Payload,
    Transform,
    Service,
    ServiceRequest,
    ServiceResponse
};
use actix_web::{HttpRequest, Result, Error, FromRequest};
use futures::future::LocalBoxFuture;
use core::future::{ready, Ready};

#[derive(Default)]
struct CountState(RefCell<i64>);

pub struct CounterMiddleware<S> {
    count: Rc<CountState>,
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
        let count = self.count.clone();
        let service = self.service.clone();
        async move {
            let value = count.0.borrow();
            *count.0.borrow_mut() = value + 1;
            req.extention_mut()
                .insert::<CountState>(count);
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
            count: Rc::new(CountState::default()),
            service: Rc::new(service)
        }))
    }
}

pub struct RequestCount(CountState);

impl FromRequest for RequestCount {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload)
        -> Self::Future
    {
        let count = req.extensions().get::<CountState>().cloned();
        ready(count)
    }
}

impl std::ops::Deref for RequestCount {
    type Target = CountState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
} 
