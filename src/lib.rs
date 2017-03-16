
use std::marker::PhantomData;

mod cheat;
use cheat::*;

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// the command trait, this is it.
/// an object which can execute a routine in an given context.
/// the context is not keept inside of the command to avoid long borrowing.
/// instead is passed to each command invocation.
/// The current command has a mutable reference as context, the outcome of the
/// command should be able to change the state of the application
pub trait Command<Context> {
    fn exec(&self, ctx: &mut Context);
}

/// a command which is payload aware is the one which can be queried for it.
/// the payload can be used to sort the commands
pub trait PayloadAware<Payload> {
    fn get_payload(&self) -> &Payload;
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// one flavour of the command interface is a Command which captures a closure.
/// yes, it is like a lambda, but we do not want to use a lambda to have later
/// access to the payload object. This payload can be used to sort the commands
pub struct CommandPayload<F, Context, Payload>
    where F: Fn(&mut Context, &Payload) -> ()
{
    func: F,
    payload: Payload,
    ghost: PhantomData<Context>,
}

impl<F, Context, Payload> CommandPayload<F, Context, Payload>
    where F: Fn(&mut Context, &Payload) -> ()
{
    pub fn new(func: F, payload: Payload) -> CommandPayload<F, Context, Payload> {
        CommandPayload {
            func: func,
            payload: payload,
            ghost: PhantomData,
        }
    }
}

impl<F, Context, Payload> Command<Context> for CommandPayload<F, Context, Payload>
    where F: Fn(&mut Context, &Payload) -> ()
{
    fn exec(&self, x: &mut Context) {
        (self.func)(x, &self.payload);
    }
}

impl<F, Context, Payload> PayloadAware<Payload> for CommandPayload<F, Context, Payload>
    where F: Fn(&mut Context, &Payload) -> ()
{
    fn get_payload(&self) -> &Payload {
        &self.payload
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// it assist us to generate the right syntax for the commands.
/// the result is a new instance of a command
macro_rules! command{
    (ctx_type : $ctx:ty => $id1:ident, execute : $body:stmt) =>
                {CommandPayload::new(move |$id1: &mut $ctx, _ |{$body}, ())};
    (ctx_type : $ctx:ty => $id1:ident, payload:$payload:expr => $id2:ident, execute : $body:stmt) => 
                {CommandPayload::new(move |$id1: &mut $ctx, ref $id2|{$body}, $payload)};
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// a basic command list, this is the first of the data "queues"
/// it preserves the order of insertion, and allows commands with different payload.
/// for this reason commands need to be wrapped in a Box.
pub struct CommandList<Ctx> {
    list: Vec<Box<Command<Ctx>>>,
}

impl<Ctx> CommandList<Ctx> {
    pub fn new() -> CommandList<Ctx> {
        CommandList { list: Vec::new() }
    }

    pub fn add(&mut self, cmd: Box<Command<Ctx>>) {
        self.list.push(cmd);
    }
}

impl<Ctx> IntoIterator for CommandList<Ctx> {
    type Item = Box<Command<Ctx>>;
    type IntoIter = ::std::vec::IntoIter<Box<Command<Ctx>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.into_iter()
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub struct CommandQueue<Ctx, K, P>
where K : Ord
{
    ghost1: PhantomData<Ctx>,
    ghost2: PhantomData<K>,
    ghost3: PhantomData<P>,
    order_f: Box<Fn(&P) -> K>,
}

impl<Ctx, K, P> CommandQueue<Ctx, K, P>
where K : Ord
{
// TODO: we pass a function, which will convert payloads into codes, which are oredered
    pub fn new<F>(f:F) -> CommandQueue<Ctx, K, P>
    where F : Fn(&P) -> K  + 'static

    {
        CommandQueue{
            ghost1: PhantomData,
            ghost2: PhantomData,
            ghost3: PhantomData,
            order_f: Box::new(f),
        }
    }

// TODO: this one needs to get the payload and create a key, priority or whatever we want to call it
    pub fn add(&mut self, _: Box<Command<Ctx>>){
        //self.queue.insert(self.order_f(
    }
}

//pub struct CQIterator<Ctx,K,P>
//where K : Ord
//{
//    ghost1: PhantomData<Ctx>,
//}
//
//impl<Ctx,K,P> CQIterator<Ctx,K,P> 
//where K : Ord
//{
//}
//
//impl<Ctx,K,P> Iterator for CQIterator<Ctx,K,P> 
//where K : Ord
//{
//    type Item = Box<Command<Ctx>>;
//    fn next(&mut self) -> Option<Box<Command<Ctx>>> {
//		None
//    }
//}
//
//impl<Ctx, K, P>  IntoIterator for CommandQueue<Ctx, K, P>
//where K : Ord
//{
//    type Item = Box<Command<Ctx>>;
//    type IntoIter = CQIterator<Ctx, K, P>;
//
//    fn into_iter(self) -> Self::IntoIter {
//		CQIterator::new(self.queue.iter())
//    }
//}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command() {

        let mut x = 12 as u32;
        let y = 1 as u32;

        let aux = CommandPayload::new(move |x: &mut u32, &y| {
                                          *x = *x + y;
                                          println!("{}", x);
                                      },
                                      y);
        aux.exec(&mut x);
        assert_eq!(x, 13);
    }

    #[test]
    fn complex_types() {

        let mut x = vec![1, 2, 3, 4];
        let y = 1;

        let aux = CommandPayload::new(move |x: &mut Vec<i32>, &y| { x.push(y); }, y);

        aux.exec(&mut x);
        assert_eq!(x.len(), 5);
    }

    struct Tmp {
        a: i32,
    }
    impl Tmp {
        fn new(a: i32) -> Tmp {
            Tmp { a: a }
        }
        fn get(&self) -> i32 {
            self.a
        }
    }

    #[test]
    fn custom_type() {

        let mut x = vec![1, 2, 3, 4];
        let y = Tmp::new(1);

        let cmd = CommandPayload::new(move |x: &mut Vec<i32>, ref y| { x.push(y.get()); }, y);


        let boxed = Box::new(cmd);
        boxed.exec(&mut x);

        assert_eq!(x.len(), 5);

    }

    #[test]
    fn mix_different() {

        let mut ctx: Vec<String> = Vec::new();
        let a = 1111 as u32;

        let cmd1 = CommandPayload::new(move |x: &mut Vec<String>, &y| { x.push(format!("{}", y)); },
                                       a);

        let b = 1.001 as f32;
        let cmd2 = CommandPayload::new(move |x: &mut Vec<String>, &y| { x.push(format!("{}", y)); },
                                       b);

        let cmd3 = CommandPayload::new(move |x: &mut Vec<String>, _| {
                                           x.push(String::from("this is text"));
                                       },
                                       ());

        let mut list: Vec<Box<Command<Vec<String>>>> = Vec::new();

        list.push(Box::new(cmd1));
        list.push(Box::new(cmd2));
        list.push(Box::new(cmd3));

        for cmd in list {
            cmd.exec(&mut ctx);
        }
        assert_eq!(ctx.len(), 3);

        for s in ctx {
            println!("{}", s);
        }
    }

    #[test]
    fn macros() {
        let mut u = 1 as u32;
        let cmd1 = command!(ctx_type:u32 => ctx, 
                            execute:println!("hello macro {}", ctx));
        cmd1.exec(&mut u);

        let cmd2 = command!(ctx_type:u32 => myctx, execute:{
            let x = 324;
            println!("hello macro, {} {}", myctx, x)
        });
        cmd2.exec(&mut u);

        let a = 1;
        let b = 1.1;
        let cmd3 = command!(ctx_type:u32 => ctx, 
                            payload:(a,b) => pay, 
                            execute:{println!("hello {:?} {}", pay, ctx);});
        cmd3.exec(&mut u);
    }

    #[test]
    fn payload() {
        let mut u = 1 as u32;
        let a = 1;
        let b = 1.1;
        let cmd = command!(ctx_type:u32 => ctx, 
                           payload:(a,b) => pay, 
                           execute: println!("hello {:?} {}", pay, ctx) );
        cmd.exec(&mut u);
        let &(x, y) = cmd.get_payload();
        assert_eq!(x, 1);
        assert_eq!(y, 1.1);
    }


    fn generate_command_list() -> CommandList<Vec<String>> {
        type Ctx = Vec<String>;

        let mut list = CommandList::<Vec<String>>::new();

        list.add(Box::new(command!( ctx_type:Ctx => ctx,
                                     execute: ctx.push(format!("hello")))));
        list.add(Box::new(command!( ctx_type:Ctx => ctx,
                                    payload: 1 => n,
                                     execute: ctx.push(format!("{}",n)))));
        list.add(Box::new(command!( ctx_type:Ctx => ctx,
                                    payload: 2 => n,
                                     execute: ctx.push(format!("{}",n)))));
        list.add(Box::new(command!( ctx_type:Ctx => ctx,
                                     payload: 3 => n,
                                      execute: ctx.push(format!("{}",n)))));
        list
    }

    #[test]
    fn command_list() {

        let mut ctx: Vec<String> = Vec::new();

        let list = generate_command_list();

        for cmd in list {
            cmd.exec(&mut ctx);
        }

        for s in ctx {
            println!("{}", s);
        }
    }

    #[test]
    fn command_queue() {

        let ctx = 1;
        let mut cq = CommandQueue::<i32, i32, i32>::new(move|x| *x);

        cq.add(Box::new(command!(ctx_type: i32 => ctx,
                        payload: 1 => i,
                        execute: println!("{}:{}", ctx, i) )));
        cq.add(Box::new(command!(ctx_type: i32 => ctx,
                        payload: 2 => i,
                        execute: println!("{}:{}", ctx, i) )));
        cq.add(Box::new(command!(ctx_type: i32 => ctx,
                        payload: 3 => i,
                        execute: println!("{}:{}", ctx, i) )));
        cq.add(Box::new(command!(ctx_type: i32 => ctx,
                        payload: 4 => i,
                        execute: println!("{}:{}", ctx, i) )));
       // for cmd in cq {
       //     cmd.exec(&mut ctx);
       // }
    }
}
