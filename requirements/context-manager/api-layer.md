# API layer — External GraphQL API

The CM's **surface**: a single endpoint speaking **GraphQL**. It is the uniform
entry for every client — requests come in here, responses go out here, and the
surface treats every caller the same.

It **serves one schema**, assembled from the entity layers (identity + domain —
[core-entity-layer.md](core-entity-layer.md),
[domain-entity-layer.md](domain-entity-layer.md)). The entities **define** the
schema; the surface only **exposes** it.

- **Queries and mutations** travel over the endpoint as request / response.
- **Subscriptions** ride a **WebSocket** the surface owns, fed by the change feed
  ([change-feed-layer.md](change-feed-layer.md)). Before broadcasting an event to a
  socket, the surface checks that **socket's token** with PermManager and pushes
  the events that subscriber may see ([permission-layer.md](permission-layer.md)).

A request entering here passes down the layers — authentication
([auth-layer.md](auth-layer.md)), then the entities with their Guards
([permission-layer.md](permission-layer.md)). The full path is the request flow in
[architecture.md](architecture.md).
