
use bamboo::{BambooHandler, BambooResult};
use request::Request;
use response::Response;

use recognizer::Router as Recognizer;
use recognizer::{Match, Params};

pub struct Router {
    router_builder:  Recognizer<Box<BambooHandler>>,
    before_handle: Box<Fn(&mut Request)>,
    after_handle: Box<Fn(&mut Response)>,
}

impl Router {
   
    // constructor
    pub fn new() -> Router {
        Router {
            router_builder: Recognizer::new(),
            before_handle: Box::new(|_: &mut Request|{}),
            after_handle: Box::new(|_: &mut Response|{})
        }
    }
    
    pub fn new_with_middleware(before_handle: Box<Fn(&mut Request)>, after_handle: Box<Fn(&mut Response)>) -> Router {
        Router {
            router_builder: Recognizer::new(),
            before_handle: before_handle,
            after_handle: after_handle
        }
    }

    // use this method to add url pattern
    fn add<H: BambooHandler> (&mut self, pattern: &str, handler: H)
        -> &mut Router {
        
        self.router_builder.add(pattern, Box::new(handler) as Box<BambooHandler>);
        // return self to trailing style expression
        self
    }

    // this method to recognize the path by previously added patterns
    fn recognize(&self, path: &str) -> Option<Match<Box<BambooHandler>>> {
        self.router_builder.recognize(path).ok()
    }
   
    // here, Request is Bamboo Request
    fn execute(&self, path: &str, req: &mut Request, res: &mut Response) -> BambooResult<String> {
        let matched = match self.recognize(path) {
            Some(matched) => matched,

            // No match
            None => return Err()
        };

        // here, we need to extract matched.params and dump them into req.params
        for (k, v) in matched.params.map.iter() {
            req.params.insert(k, v);
        }

        // execute the truely function handler
        // corresponding to hyper
        matched.handler.handle(req, res)

    }

}

unsafe impl Send for Router {}
unsafe impl Sync for Router {}

// implement this, make Router become a acceptable handler
impl BambooHandler for Router {
    
    fn handle(&self, req: &mut Request, res: &mut Response) -> BambooResult<String> {
        // before from Middleware trait
        let mut ret = (self.before_handle)(req);
        match ret {
            Ok(_) => {
                // main execution
                let path = req.uri.path;
                ret = self.execute(path, req, res);
                match ret {
                    Ok(body) => {
                        // after from Middleware trait
                        (self.after_handle)(res);
                        // once handler produce body, post middleware couldn't modify it?
                        body
    
                    },
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }

}

