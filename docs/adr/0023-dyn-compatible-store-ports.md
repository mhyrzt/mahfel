# ADR 0023: Dyn-compatible store ports at the composition boundary

## Status

Accepted

## Context

Application ports (`RoomStore`, `SessionStore`, `UserStore`, `IdentityStore`,
`ServicePrincipalStore`, `CoordinationStore`, `EventPublisher`, `MediaGateway`,
`UserAttributeWriter`) were declared with return-position `impl Future` (RPITIT). RPITIT is not
dyn-compatible, so a port could not be stored as a trait object.

The API transport needs to hold these ports in one shared `AppState`. Because the ports were not
dyn-compatible, the transport bound its router state to one concrete store adapter through a generic
parameter:

```rust
pub trait RoomStoreState: RoomStore + Clone + Send + Sync + 'static {}
pub struct AppState<R: RoomStoreState> {
    pub rooms: R,
    pub sessions: SessionStoreRef, // Arc<dyn ...> seam
    ...
}
```

Sessions arrived second, and rather than add a second generic parameter to every route signature,
they were placed behind a hand-written object-safe seam: a mirror trait (`SessionStoreHandle`) that
redeclared every method returning `Pin<Box<dyn Future>>`, a blanket impl that boxed each call, and an
adapter (`SessionStoreRef`) that re-implemented `SessionStore` to delegate back across the seam.
That was roughly 95 lines to hold one extra store.

The seam did not generalize. Milestone 4 adds at least two more stores (session participants for
M4-02, a block list for M4-05). Each one would either copy the mirror-trait pattern or add another
generic parameter to the routes. Both outcomes were rejected: the first duplicates the same adapter
by hand per store, and the second multiplies the generic surface with every store and is invasive
across every handler, the router merge chain, the WebSocket module, and all test fixtures.

A probe confirmed the intended shape compiles: a trait whose methods return an explicit
`Pin<Box<dyn Future<Output = T> + Send + 'a>>` is dyn-compatible, and a concrete adapter satisfies it
by writing `fn m(&self) -> BoxFuture<'_, T> { Box::pin(async move { ... }) }`. Generic use-case
functions also accept such a port unchanged.

## Decision

Application ports return a boxed future and are dyn-compatible. The composition boundary holds one
`Arc<dyn Port>` per capability instead of a generic parameter per store.

- A shared alias lives in `mahfel-application::ports`:

  ```rust
  pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
  ```

- Every port method returns `BoxFuture`. A port method that borrows any argument besides `&self`
  declares an explicit lifetime tying all borrowed inputs to the returned future
  (`fn get<'a>(&'a self, key: &'a str) -> BoxFuture<'a, T>`). Eliding to `BoxFuture<'_, T>` does not
  compile when more than one reference is borrowed, because `'_` binds to a single anonymous input.
- The API `AppState` is a flat struct with no store generic:

  ```rust
  pub struct AppState {
      pub rooms: Arc<dyn RoomStore>,
      pub sessions: Arc<dyn SessionStore>,
      ...
  }
  ```

  `RoomStoreState`, `SessionStoreHandle`, and `SessionStoreRef` are removed. Adding a store is a new
  field, not a new type parameter threaded through route signatures.
- Application use cases stay generic over the port and add `?Sized` (`R: RoomStore + ?Sized`) so they
  accept both a concrete adapter and `&dyn Port`. Their behavior is unchanged.
- The boxing cost is paid uniformly at the port boundary. This is the same cost the session seam
  already paid; the decision makes that cost consistent rather than exceptional.

This supersedes the earlier `backend/AGENTS.md` guidance that ports should return `impl Future` and
that the composition surface should use a generic parameter instead of boxing futures. That guidance
is replaced by the rules above.

## Consequences

- One adapter shape exists instead of a mirror trait plus adapter per store. The session seam and its
  ~95 lines are deleted, and M4-02 and M4-05 add a store as a plain `Arc<dyn ...>` field.
- Route handlers, the router merge chain, the WebSocket module, and the extractors lose their store
  generic parameter; the resulting code is flatter.
- Each port call boxes its future, exactly as the removed session seam did. The allocation is at the
  composition boundary, not in the domain or application logic.
- Adapter implementations write `fn ... -> BoxFuture { Box::pin(async move { ... }) }` rather than
  `async fn`. The bodies are unchanged; the signature is not. Test doubles must follow the same
  shape.
- Genericity is retained where it carries meaning: use-case functions remain generic over their
  ports, so a unit test can still supply a concrete in-memory fake without a trait object.
- The in-process WebSocket registry remains process-local and unaffected; this decision concerns
  store ports only, not ephemeral coordination or fan-out.

## Alternatives considered

- **Keep the generic parameter per store, one parameter for each.** Rejected: it grows with every
  store, is invasive across every handler signature and the router merge chain, and was actively
  painful to apply. It also does not remove the asymmetry, it entrenches it.
- **Keep the hand-written object-safe seam and copy it per store.** Rejected: the mirror trait, the
  blanket impl, and the adapter are the same code repeated per store and must be kept in sync by
  hand. This is what the session seam already was; copying it two more times is the problem, not the
  solution.
- **Bundle all stores into one struct that implements every port by delegating to its own fields.**
  Rejected: it invents a bundle that claims to be each port (a `PostgresStores: RoomStore` that is not
  conceptually a room store), and the delegation is boilerplate that duplicates the port surface. It
  was attempted and discarded.
- **Return `impl Future` and box only where the transport needs it.** Rejected: this is the current
  state, and it produces one bespoke seam per store that needs to cross the boundary. The asymmetry
  is the defect.
