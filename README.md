# poll_until_done

A simple Rust crate that polls a future until it is done. This helps handle the 
[function coloring problem](https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/) 
by replacing `my_async_function().await` with `my_async_function().run()`, which has the same behavior 
of blocking until the future finishes running, but without requiring the caller to be `async`.
