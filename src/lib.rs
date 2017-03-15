use std::marker::PhantomData;


// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// the command trait, this is it.
/// an object which can execute a routine in an given context.
/// the context is not keept inside of the command to avoid long borrowing.
/// instead is passed to each command invocation.
/// The current command has a mutable reference as context, the outcome of the
/// command should be able to change the state of the application
pub trait Command<Context>{
    fn exec(&self, ctx : &mut Context);
}

/// a command which is payload aware is the one which can be queried for it.
/// the payload can be used to sort the commands
pub trait PayloadAware<Payload>{
    fn get_payload(&self) -> &Payload;
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// one flavour of the command interface is a Command which captures a closure.
/// yes, it is like a lambda, but we do not want to use a lambda to have later
/// access to the payload object. This payload can be used to sort the commands
pub struct CommandPayload<F, Context, Payload>
where F : Fn(&mut Context, &Payload)  -> ()
{
   func : F,
   payload: Payload,
   ghost: PhantomData<Context>,
}

impl<F, Context, Payload> CommandPayload<F, Context, Payload>
where F : Fn(&mut Context, &Payload)  -> () {
    fn new (func : F, payload: Payload) -> CommandPayload<F, Context, Payload> {
        CommandPayload{
            func: func,
            payload: payload,
            ghost:  PhantomData,
        }
    }
}

impl<F, Context, Payload> Command<Context> for CommandPayload<F, Context, Payload>
where F : Fn(&mut Context, &Payload)  -> () {
    fn exec (&self, x : &mut Context){
        (self.func)(x, &self.payload);
    }
}

impl<F, Context, Payload> PayloadAware<Payload> for CommandPayload<F, Context, Payload> 
where F : Fn(&mut Context, &Payload)  -> () {
    fn get_payload(&self) -> &Payload {
        &self.payload
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// another flavour of command is one with no payload. This one keeps a closure
/// and we have no introspection on the state it capures.
struct CommandNoPayload<F, Context>
where F : Fn(&mut Context)  -> () {
   func : F,
   ghost: PhantomData<Context>,
}

impl<F, Context> CommandNoPayload<F, Context>
where F : Fn(&mut Context)  -> () {
    fn new (func : F) -> CommandNoPayload<F, Context> {
        CommandNoPayload{
            func: func,
            ghost:  PhantomData,
        }
    }
}

impl<F, Context> Command<Context> for CommandNoPayload<F, Context>
where F : Fn(&mut Context)  -> () {
    fn exec (&self, x : &mut Context){
        (self.func)(x);
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// it assist us to generate the right syntax for the commands.
macro_rules! command{
    (ctx_type : $ctx:ty => $id1:ident, execute : $body:stmt) => {CommandNoPayload::new(move |$id1: &mut $ctx|{$body})};
    (ctx_type : $ctx:ty => $id1:ident, payload:$payload:expr => $id2:ident, execute : $body:stmt) => 
                {CommandPayload::new(move |$id1: &mut $ctx, ref $id2|{$body}, $payload)};
}

/// another helper to generate Boxes for the commands
macro_rules! command_box{
    (ctx_type : $ctx:ty => $id1:ident, execute : $body:stmt) => {Box::new(command!(ctx_type:$ctx => $id1, execute: $body))};
    (ctx_type : $ctx:ty => $id1:ident, payload:$payload:expr => $id2:ident, execute : $body:stmt) => 
            {Box::new(command!(ctx_type:$ctx => $id1, 
                               payload:$payload => $id2,
                               execute: $body))};
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
struct CommandQueue{
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
struct Pipeline{
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command() {

        let mut x = 12 as u32;
        let y = 1 as u32;

        let aux = CommandPayload::new(move |x: &mut u32, &y| {
            *x = *x+y;
            println!("{}", x);
        }, y);
        aux.exec(&mut x);
        assert_eq!(x, 13);
    }

    #[test]
    fn complex_types() {

        let mut x = vec!(1,2,3,4);
        let y = 1;

        let aux = CommandPayload::new(move |x: &mut Vec<i32>, &y| {
            x.push(y);
        }, y);

        aux.exec(&mut x);
        assert_eq!(x.len(), 5);
    }

    struct Tmp{
        a: i32,
    }
    impl Tmp {
        fn new (a: i32) -> Tmp{
            Tmp{
                a: a,
            }
        }
        fn get(&self) -> i32
        {
            self.a
        }
    }

    #[test]
    fn custom_type() {

        let mut x = vec!(1,2,3,4);
        let y = Tmp::new(1);

        let cmd = CommandPayload::new(move |x: &mut Vec<i32>, ref y| {
            x.push(y.get());
        }, y);


        let boxed = Box::new(cmd);
        boxed.exec(&mut x);

        assert_eq!(x.len(), 5);

    }

    #[test]
    fn mix_different() {

        let mut ctx : Vec<String> = Vec::new();
        let a = 1111 as u32;

        let cmd1 = CommandPayload::new(move |x: &mut Vec<String>, &y| {
            x.push(format!("{}", y));
        }, a);

        let b = 1.001 as f32;
        let cmd2 = CommandPayload::new(move |x: &mut Vec<String>, &y| {
            x.push(format!("{}", y));
        }, b);


        let cmd3 = CommandNoPayload::new(move |x: &mut Vec<String>| {
            x.push(String::from("this is text"));
        });

        let mut list : Vec<Box<Command<Vec<String>>>> = Vec::new();

        list.push(Box::new(cmd1));
        list.push(Box::new(cmd2));
        list.push(Box::new(cmd3));

        for cmd in list{
            cmd.exec(&mut ctx);
        }
        assert_eq!(ctx.len(), 3);

        for s in ctx{
            println!("{}", s);
        }
    }

    #[test]
    fn macros() {
        let mut u = 1 as u32;
        let cmd1 = command!(ctx_type:u32 => ctx, execute:println!("hello macro"));
        cmd1.exec(&mut u);

        let cmd2 = command!(ctx_type:u32 => myctx, execute:{
            let x = 324;
            println!("hello macro, {} {}", myctx, x)
        });
        cmd2.exec(&mut u);

        let a = 1;
        let b = 1.1;
        let cmd3 = command!(ctx_type:u32 => ctx, payload:(a,b) => pay, execute:{println!("hello {:?}", pay);});
        cmd3.exec(&mut u);
    }

    #[test]
    fn macros_box() {
        let mut u = 1 as u32;
        let cmd1 = command_box!(ctx_type:u32 => ctx, execute:println!("hello macro"));
        cmd1.exec(&mut u);

        let cmd2 = command_box!(ctx_type:u32 => myctx, execute:{
            let x = 324;
            println!("hello macro, {} {}", myctx, x)
        });
        cmd2.exec(&mut u);

        let a = 1;
        let b = 1.1;
        let cmd3 = command_box!(ctx_type:u32 => ctx, payload:(a,b) => pay, execute:{println!("hello {:?}", pay);});
        cmd3.exec(&mut u);
    }
    #[test]
    fn payload() {
        let mut u = 1 as u32;
        let a = 1;
        let b = 1.1;
        let cmd = command!(ctx_type:u32 => ctx, 
                           payload:(a,b) => pay, 
                           execute: println!("hello {:?}", pay) );
        cmd.exec(&mut u);
        let &(x,y) = cmd.get_payload();
        assert_eq!(x, 1);
        assert_eq!(y, 1.1);
    }
}
