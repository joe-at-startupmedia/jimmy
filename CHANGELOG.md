# 0.1.0 (2021-11-13)

* Forked from Ocypod with merged features

# 0.2.0 
* Merged the following updates from upstream:
* Updated to use Actix web 4.x (beta), Tokio 1.x.
* Switch to using deadpool to manage async Redis connections, to avoid possible
  race conditions with transactions over a multiplexed connection (see
    [#26](https://github.com/davechallis/ocypod/issues/26).
* More consistent HTTP status codes returned on error.
