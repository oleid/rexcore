/// Recursively call callables, which implement some Fn trait
/// ```
/// # #[macro_use] extern crate rexcore;
/// # fn main() {
/// let f = chain!( |v| 2*v =>> |u| u-1 =>> |x| x*x);
/// assert_eq!( f(2), 9);
/// # }
/// ```
#[macro_export]
macro_rules! chain {
    // -- call $dest with the result of $src
    ( $src:expr =>> $dest:expr) => {
        {
            // we need to assign the closures to values, before
            // creating the new closure, in order to make sink!
            // work. Otherwise the entire $node is moved into the lambda.
            let f1 = $src;
            let f2 = $dest;

            move |v| f2(f1(v))
        }
    };
    // --- Handle the rest recursively
    ($src:expr =>> $dest:expr  $(=>> $dests:expr)+ ) => {
        chain! {
            chain! { $src =>> $dest }
            $(=>> $dests )+
        }
    };
}

/// Connect event sink with a source, possibly chaining intermediate connections
/// An Event Sink is either something created with the sink! macro, or
/// a lambda
/// ```
/// # #[macro_use] extern crate rexcore;
/// # use rexcore::EventSource;
/// # fn main() {
/// let mut e = EventSource::new();
/// connect!(e
///     =>> |v| 2*v =>> |u| u-1 =>> |x| x*x
///     =>> |s| println!("Outch, got a {}", s)
/// );
/// e.fire(2);
/// # }
/// ```
#[macro_export]
macro_rules! connect {
    // Use the above's chain macro to handle multiple parameters
    ($src:expr  =>> $trans1:expr $(=>> $transes:expr)+ ) => {{
        connect!( $src =>>  chain!($trans1 $(=>> $transes)+ ) )
    }};
    ( $src:expr =>> $dest:expr ) => {
        $src.push( Box::new($dest));
    };
}
