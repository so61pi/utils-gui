## Development

Run the following command in the root of your project to start developing with the default platform:

```sh
# OpenSUSE may need this
export WEBKIT_DISABLE_COMPOSITING_MODE=1

dx serve
```

To convert a HTML snippet to RSX syntax, put the HTML content to `./tmp/test.html` then run below command.

```sh
dx translate -f ./tmp/test.html -o ./tmp/test.html.rsx
```

## Dioxus 0.6 Knowledge

- Signals will only subscribe to components when you read from the signal in that component.
  + https://github.com/DioxusLabs/dioxus/blob/v0.6/packages/signals/README.md?#local-subscriptions
- `Signal` is implemented with [generational-box](https://crates.io/crates/generational-box).
  + A generational-box is used to store multiple values.
  + Internally, a generational-box contains an array of slots. Each slot holds (1) a generational id which is a unique value indicating the generation of that slot, and (2) a pointer to stored data.
  + To reference to a stored data, we need (1) the index of the array slot and (2) the generational id. This is how `Signal` keeps its stored data. This data is `Copy`able, and all copies point to a same underlying value.
  + The `PartialEq` implementation of `Signal` basically checks the pointer of the stored value, not the content.
  + Read more about the idea of generational arena at https://github.com/fitzgen/generational-arena
- The value of signal does not need to be `PartialEq`. The state tracking is based on the `<signal>.write()` function. When the return value of `<signal>.write()` function is dropped, the subscribers of that signal (i.e., components) are marked as dirty, thus triggering rerun.
- The closure passed to `use_memo` will be called whenever the inside signal is changed (i.e., due to `<signal>.write()`). The `use_memo` then decide whether to rerun the component based on the comparison result using `PartialEq`.
- https://users.rust-lang.org/t/understanding-dioxus-signals-or-state-management-in-general/111611/4
- `signal.read()` only reads, `signal()` is `signal.read().clone()`.
- Avoid calling callback/event handler in reactive scopes (component, use_memo...) as it would trigger infinite loop since the callback/event handler can read & write to a same signal.

### Async

- Signal works similar to `RefCell` in regards to borrowing. That means mut borrowing is checked at runtime. This can create a situation in which a signal is mut borrowed while is already borrowed elsewhere, thus triggering panic. This easily happens with async code as the borrowed value is dropped at the end of statement.

```rust
async fn do_something(v: Vec<i32>) {
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
}

fn Component {
  let v = use_signal(|| vec![]);

  use_effect(move || async move {
      // Borrow `v` with `v.peek()` which creates a temporary that is dropped at
      // the **end** of await statement. So the `v.write` might run while this
      // statement is awaited by async runtime.
      do_something(v.peek().clone()).await;
  });

  use_effect(move || async move {
      // The solution is to move `v.peek` to its own statement.
      let copy = v.peek().clone()
      do_something(copy).await;
  });

  rsx! {
      button {
          onclick: move || async move {
              // Mut borrow to modify.
              v.write().push(1);
          },
          "Button"
      }
  }
}
```

More can be read at:
- https://stackoverflow.com/questions/75586097/temporary-value-lifetime-in-async-rust
- https://rustwiki.org/en/reference/destructors.html#temporary-scopes
